use std::cell::Cell;

use bitflags::bitflags;
use micropython_rs::obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags};

use super::modvasyncio::event_loop::{self, EventLoop};
use crate::qstrgen::qstr;

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
pub struct CompetitionRuntime {
    base: ObjBase<'static>,

    // Dragon Ball Reference (Cell)
    status: Cell<Status>,
    phase: Cell<Phase>,

    /// Arbitrary user class that may contain the following competition methods:
    /// - async def connected(self)
    /// - async def disconnected(self)
    /// - async def driver(self)
    /// - async def autonomous(self)
    /// - async def disabled(self)
    /// Any absent methods are replaced with noops.
    robot_obj: Obj,

    // nullable
    coro: Cell<Obj>,
}

pub static COMPETITION_RUNTIME_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(CompetitionRuntime))
        .set_iter(Iter::IterNext(runtime_iternext));

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
                .tick_coro(Obj::NULL, self.coro.get(), Obj::NONE);

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
            self.coro.set(match self.phase.get() {
                Phase::Connected => todo!(),
                Phase::Disconnected => todo!(),
                Phase::Mode(_) => todo!(),
            });
        }
    }
}

extern "C" fn runtime_iternext(self_in: Obj) -> Obj {
    self_in.try_as_obj::<CompetitionRuntime>().unwrap().tick();
    Obj::NONE
}
