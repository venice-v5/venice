use std::cell::Cell;

use bitflags::bitflags;
use micropython_rs::{
    const_dict,
    except::raise_type_error,
    generator::GEN_INSTANCE_TYPE,
    init::token,
    make_new_from_fn,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use crate::{
    args::ArgType,
    error_msg::error_msg,
    fun::fun1,
    modvasyncio::event_loop::{self, EventLoop},
    obj::alloc_obj,
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
        match self {
            Self::Connected | Self::Disconnected => false,
            _ => true,
        }
    }
}

#[repr(C)]
pub struct Competition {
    base: ObjBase<'static>,

    connected: Obj,
    disconnected: Obj,
    driver: Obj,
    autonomous: Obj,
    disabled: Obj,
}

#[repr(C)]
pub struct CompetitionRuntime {
    base: ObjBase<'static>,

    // Dragon Ball Reference (Cell)
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

pub static COMPETITION_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Competition))
        .set_make_new(make_new_from_fn!(competition_make_new))
        .set_locals_dict(const_dict![
            qstr!(run) => Obj::from_static(&fun1!(competition_run, &Competition)),
        ]);

pub static COMPETITION_RUNTIME_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(CompetitionRuntime))
        .set_iter(Iter::IterNext(runtime_iternext));

unsafe impl ObjTrait for Competition {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = COMPETITION_RUNTIME_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for CompetitionRuntime {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = COMPETITION_RUNTIME_OBJ_TYPE.as_obj_type();
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

        match self.poll_update() {
            Some(prev_status) => {
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
            None => {}
        }

        if !self.coro.get().is_null() {
            // tick the coroutine on the current task
            let terminated = event_loop::get_running_loop()
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

fn competition_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    if n_pos > 0 {
        raise_type_error(token(), c"function does not accept positional arguments");
    }

    let mut comp = Competition {
        base: ObjBase::new(ty),

        connected: Obj::NULL,
        disconnected: Obj::NULL,
        driver: Obj::NULL,
        autonomous: Obj::NULL,
        disabled: Obj::NULL,
    };

    for i in 0..n_kw {
        let k = args[i * 2].get_str().unwrap();
        let v = args[i * 2 + 1];

        let routine = match k {
            b"driver" => &mut comp.driver,
            b"autonomous" => &mut comp.autonomous,
            b"connected" => &mut comp.connected,
            b"disconnected" => &mut comp.disconnected,
            b"disabled" => &mut comp.disabled,
            _ => raise_type_error(
                token(),
                error_msg!(
                    "no such competition routine '{}'",
                    // python strings are utf8, right?
                    str::from_utf8(k).unwrap()
                ),
            ),
        };

        if !v.is_callable() {
            raise_type_error(
                token(),
                error_msg!(
                    "{} routine value is not callable",
                    str::from_utf8(k).unwrap()
                ),
            );
        }
        *routine = v;
    }

    alloc_obj(comp)
}

fn competition_run(comp: &Competition) -> Obj {
    alloc_obj(CompetitionRuntime {
        base: ObjBase::new(CompetitionRuntime::OBJ_TYPE),

        status: Cell::new(status()),
        // TODO: maybe this should be made an option since we haven't computed the phase yet
        phase: Cell::new(Phase::Disconnected),

        connected: comp.connected,
        disconnected: comp.disconnected,
        driver: comp.driver,
        autonomous: comp.autonomous,
        disabled: comp.disabled,

        coro: Cell::new(Obj::NULL),
    })
}

extern "C" fn runtime_iternext(self_in: Obj) -> Obj {
    self_in.try_as_obj::<CompetitionRuntime>().unwrap().tick();
    Obj::NONE
}
