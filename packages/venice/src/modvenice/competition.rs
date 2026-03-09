use std::cell::Cell;

use argparse::{ArgType, error_msg};
use bitflags::bitflags;
use micropython_rs::{
    class, class_methods,
    except::raise_type_error,
    fun::Fun2,
    generator::GEN_INSTANCE_TYPE,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};

use crate::{
    modvenice::vasyncio::event_loop::{EventLoop, vasyncio_get_running_loop},
    qstrgen::qstr,
};

bitflags! {
    // thanks for the comments vexide
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Status: u32 {
        /// Robot is disabled by field control.
        const DISABLED = 1 << 0;
        /// Robot is in autonomous mode.
        const AUTONOMOUS = 1 << 1;
        /// Robot is connected to competition control.
        const CONNECTED = 1 << 2;
        /// Robot is connected to field control (NOT competition switch).
        const SYSTEM = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Driver,
    Autonomous,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Connected,
    Disconnected,
    Mode(Mode),
}

pub fn status() -> Status {
    Status::from_bits_retain(unsafe { vex_sdk::vexCompetitionStatus() })
}

impl Status {
    pub const fn connected(self) -> bool {
        self.contains(Status::CONNECTED)
    }

    pub const fn mode(self) -> Mode {
        if self.contains(Status::DISABLED) {
            Mode::Disabled
        } else if self.contains(Status::AUTONOMOUS) {
            Mode::Autonomous
        } else {
            Mode::Driver
        }
    }
}

impl Phase {
    pub const fn interruptable(self) -> bool {
        !matches!(self, Self::Connected | Self::Disconnected)
    }
}

#[class(qstr!(Competition))]
#[repr(C)]
pub struct Competition {
    base: ObjBase<'static>,

    connected: Cell<Obj>,
    disconnected: Cell<Obj>,
    driver: Cell<Obj>,
    autonomous: Cell<Obj>,
    disabled: Cell<Obj>,
}

#[class(qstr!(CompetitionRuntime))]
#[repr(C)]
pub struct CompetitionRuntime {
    base: ObjBase<'static>,

    // Dragon Ball Reference (Cell)
    // low level larping
    status: Cell<Status>,
    phase: Cell<Phase>,

    connected: Obj,
    disconnected: Obj,
    driver: Obj,
    autonomous: Obj,
    disabled: Obj,

    // nullable
    coro: Cell<Obj>,
}

impl CompetitionRuntime {
    pub fn poll_update(&self) -> Option<Status> {
        let new_status = status();
        let prev_status = self.status.replace(new_status);

        if prev_status != new_status {
            Some(prev_status)
        } else {
            None
        }
    }

    pub fn tick(&self) {
        let mut phase_updated = false;

        if let Some(prev_status) = self.poll_update() {
            let new_status = self.status.get();
            if !self.phase.get().interruptable() {
                self.phase
                    .set(if prev_status.connected() != new_status.connected() {
                        match new_status.connected() {
                            true => Phase::Connected,
                            false => Phase::Disconnected,
                        }
                    } else {
                        Phase::Mode(new_status.mode())
                    });

                phase_updated = true;
            }
        }

        if !self.coro.get().is_null() {
            // tick the coroutine on the current task
            let terminated = vasyncio_get_running_loop()
                .try_as_obj::<EventLoop>()
                .unwrap()
                .tick_coro(Obj::NULL, self.coro.get());

            if terminated {
                match self.phase.get() {
                    Phase::Connected | Phase::Disconnected => {
                        self.phase.set(Phase::Mode(self.status.get().mode()));
                        phase_updated = true;
                    }
                    Phase::Mode(_) => {}
                }
            }
        }

        // update coroutine
        if phase_updated {
            self.coro.set({
                let coro = match self.phase.get() {
                    Phase::Connected => self.connected,
                    Phase::Disconnected => self.disconnected,
                    Phase::Mode(Mode::Driver) => self.driver,
                    Phase::Mode(Mode::Autonomous) => self.autonomous,
                    Phase::Mode(Mode::Disabled) => self.disabled,
                }
                .call(0, &[])
                .unwrap(); // object is verified to be callable in make_new

                if !coro.is(GEN_INSTANCE_TYPE) && !coro.is_null() {
                    let phase_name = match self.phase.get() {
                        Phase::Connected => "connected",
                        Phase::Disconnected => "disconnected",
                        Phase::Mode(Mode::Driver) => "driver",
                        Phase::Mode(Mode::Autonomous) => "autonomous",
                        Phase::Mode(Mode::Disabled) => "disabled",
                    };
                    raise_type_error(
                        token(),
                        error_msg!(
                            "expected coroutine return value from {phase_name} routine, got <{}>",
                            ArgType::of(&coro)
                        ),
                    );
                }

                coro
            });
        }
    }
}

fn assert_callable(routine: Obj) {
    if !routine.is_callable() {
        raise_type_error(token(), c"routine object is not callable");
    }
}

macro_rules! routine_decorator {
    ($fn_name:ident, $routine_name:ident) => {
        extern "C" fn $fn_name(self_in: Obj, routine: Obj) -> Obj {
            let comp = self_in.try_as_obj::<Competition>().unwrap();
            assert_callable(routine);
            comp.$routine_name.set(routine);
            routine
        }
    };
}

routine_decorator!(competition_connected, connected);
routine_decorator!(competition_disconnected, disconnected);
routine_decorator!(competition_driver, driver);
routine_decorator!(competition_autonomous, autonomous);
routine_decorator!(competition_disabled, disabled);

#[class_methods]
impl Competition {
    #[make_new]
    fn make_new(ty: &'static ObjType, _n_pos: usize, _n_kw: usize, args: &[Obj]) -> Self {
        if !args.is_empty() {
            raise_type_error(token(), c"function does not accept arguments");
        }

        Self {
            base: ObjBase::new(ty),

            connected: Cell::new(Obj::NULL),
            disconnected: Cell::new(Obj::NULL),
            driver: Cell::new(Obj::NULL),
            autonomous: Cell::new(Obj::NULL),
            disabled: Cell::new(Obj::NULL),
        }
    }

    #[constant(qstr!(connected))]
    const CONNECTED: &Fun2 = &Fun2::new(competition_connected);
    #[constant(qstr!(disconnected))]
    const DISCONNECTED: &Fun2 = &Fun2::new(competition_disconnected);
    #[constant(qstr!(driver))]
    const DRIVER: &Fun2 = &Fun2::new(competition_driver);
    #[constant(qstr!(autonomous))]
    const AUTONOMOUS: &Fun2 = &Fun2::new(competition_autonomous);
    #[constant(qstr!(disabled))]
    const DISABLED: &Fun2 = &Fun2::new(competition_disabled);

    #[method]
    fn run(&self) -> CompetitionRuntime {
        CompetitionRuntime {
            base: ObjBase::new(CompetitionRuntime::OBJ_TYPE),

            status: Cell::new(status()),
            // TODO: maybe this should be made an option since we haven't computed the phase yet
            phase: Cell::new(Phase::Disconnected),

            connected: self.connected.get(),
            disconnected: self.disconnected.get(),
            driver: self.driver.get(),
            autonomous: self.autonomous.get(),
            disabled: self.disabled.get(),

            coro: Cell::new(Obj::NULL),
        }
    }
}

#[class_methods]
impl CompetitionRuntime {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
        self_in.try_as_obj::<Self>().unwrap().tick();
        Obj::NONE
    }
}
