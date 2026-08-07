use std::cell::Cell;

use argparse::{ArgType, Callable, error_msg};
use bitflags::bitflags;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::type_error,
    generator::{GEN_INSTANCE_TYPE, VmReturnKind, close_gen, resume_gen},
    init::token,
    nlr,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};

use crate::modvenice::{
    Exception,
    vasyncio::{sleep::Sleep, task::Task, time32},
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
    Initial,
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

#[derive(Clone, Copy)]
enum RoutineWait {
    Ready,
    Sleep {
        sleep: Obj,
        deadline: time32::Instant,
    },
    Task(Obj),
}

#[class(qstr!(Competition))]
#[repr(C)]
pub struct Competition {
    base: ObjBase,

    connected: Cell<Option<Callable>>,
    disconnected: Cell<Option<Callable>>,
    driver: Cell<Option<Callable>>,
    autonomous: Cell<Option<Callable>>,
    disabled: Cell<Option<Callable>>,
}

#[class(qstr!(CompetitionRuntime))]
#[repr(C)]
pub struct CompetitionRuntime {
    base: ObjBase,

    // Dragon Ball Reference (Cell)
    // low level larping
    status: Cell<Status>,
    phase: Cell<Phase>,

    connected: Option<Callable>,
    disconnected: Option<Callable>,
    driver: Option<Callable>,
    autonomous: Option<Callable>,
    disabled: Option<Callable>,

    // nullable
    coro: Cell<Obj>,
    routine_wait: Cell<RoutineWait>,
}

impl CompetitionRuntime {
    /// Returns the next competition status and phase when the active phase must change.
    ///
    /// Connected and disconnected routines are transient phases. Status changes update the stored
    /// status while one of those routines runs, but do not interrupt the routine. Interruptible
    /// phase updates are not committed until the previous routine closes successfully.
    fn next_phase(&self) -> Option<(Status, Phase)> {
        let old_phase = self.phase.get();
        let new_status = status();
        let old_status = self.status.get();

        if old_phase == Phase::Initial {
            return Some((new_status, Phase::Mode(new_status.mode())));
        }

        if old_status == new_status {
            return None;
        }

        if !old_phase.interruptable() {
            self.status.set(new_status);
            return None;
        }

        let new_phase = if old_status.connected() != new_status.connected() {
            match new_status.connected() {
                true => Phase::Connected,
                false => Phase::Disconnected,
            }
        } else {
            Phase::Mode(new_status.mode())
        };

        if old_phase == new_phase {
            self.status.set(new_status);
            None
        } else {
            Some((new_status, new_phase))
        }
    }

    fn phase_name(phase: Phase) -> &'static str {
        match phase {
            Phase::Connected => "connected",
            Phase::Disconnected => "disconnected",
            Phase::Mode(Mode::Driver) => "driver",
            Phase::Mode(Mode::Autonomous) => "autonomous",
            Phase::Mode(Mode::Disabled) => "disabled",
            Phase::Initial => unreachable!(),
        }
    }

    fn clear_phase_routine(&self) {
        self.coro.set(Obj::NULL);
        self.routine_wait.set(RoutineWait::Ready);
    }

    fn close_phase_routine(&self) {
        let coro = self.coro.replace(Obj::NULL);
        self.routine_wait.set(RoutineWait::Ready);

        if !coro.is_null() {
            close_gen(coro);
        }
    }

    fn create_phase_routine(&self, phase: Phase) -> Result<Obj, Exception> {
        let coro = match phase {
            Phase::Connected => self.connected,
            Phase::Disconnected => self.disconnected,
            Phase::Mode(Mode::Driver) => self.driver,
            Phase::Mode(Mode::Autonomous) => self.autonomous,
            Phase::Mode(Mode::Disabled) => self.disabled,
            Phase::Initial => unreachable!(),
        }
        .map(|routine| routine.call(0, &[]))
        .unwrap_or(Obj::NULL);

        if !coro.is_null() && !coro.is(GEN_INSTANCE_TYPE) {
            Err(type_error(error_msg!(
                "expected coroutine return value from {} routine, got <{}>",
                Self::phase_name(phase),
                ArgType::of(&coro)
            )))?;
        }

        Ok(coro)
    }

    fn set_phase_routine(&self, phase: Phase, coro: Obj) {
        self.phase.set(phase);
        self.coro.set(coro);
        self.routine_wait.set(RoutineWait::Ready);
    }

    /// Returns whether the active routine can be resumed without violating its current await.
    ///
    /// Competition status must remain responsive while a routine sleeps or awaits a task. The
    /// competition runtime therefore tracks those waits itself and yields a generic wake signal so
    /// its outer task remains in the event loop's ready queue.
    fn routine_ready(&self) -> bool {
        match self.routine_wait.get() {
            RoutineWait::Ready => true,
            RoutineWait::Sleep { sleep, deadline } => {
                if deadline <= time32::Instant::now() {
                    sleep.as_obj::<Sleep>().complete();
                    self.routine_wait.set(RoutineWait::Ready);
                    true
                } else {
                    false
                }
            }
            RoutineWait::Task(task) => {
                if task.as_obj::<Task>().is_complete() {
                    self.routine_wait.set(RoutineWait::Ready);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn enter_current_mode(&self) -> Result<(), Exception> {
        let phase = Phase::Mode(self.status.get().mode());
        let coro = self.create_phase_routine(phase)?;
        self.set_phase_routine(phase, coro);
        Ok(())
    }

    /// Polls the competition runtime once and returns the object that its active routine yielded.
    ///
    /// This method deliberately does not call the event loop. The yielded object must propagate
    /// through the coroutine that awaits this runtime so the event loop schedules only that outer
    /// task.
    pub fn tick(&self) -> Result<Obj, Exception> {
        if let Some((new_status, new_phase)) = self.next_phase() {
            // Close the previous phase routine before starting the new one. This injects
            // GeneratorExit so synchronous finally blocks can clean up mode-specific state.
            // Commit the new state only after cleanup and routine creation both succeed.
            self.close_phase_routine();
            let coro = self.create_phase_routine(new_phase)?;
            self.status.set(new_status);
            self.set_phase_routine(new_phase, coro);
        }

        loop {
            let coro = self.coro.get();

            if coro.is_null() {
                // Missing connected/disconnected routines behave like completed no-op routines.
                // This prevents the runtime from becoming stuck in a transient phase.
                match self.phase.get() {
                    Phase::Connected | Phase::Disconnected => {
                        self.enter_current_mode()?;
                        continue;
                    }
                    Phase::Mode(_) => return Ok(Obj::NONE),
                    Phase::Initial => unreachable!(),
                }
            }

            if !self.routine_ready() {
                return Ok(Obj::NONE);
            }

            let result = resume_gen(coro, Obj::NONE, Obj::NULL);
            match result.return_kind {
                VmReturnKind::Yield => {
                    if let Some(sleep) = result.obj.try_as_obj::<Sleep>() {
                        self.routine_wait.set(RoutineWait::Sleep {
                            sleep: result.obj,
                            deadline: time32::Instant::now() + sleep.duration(),
                        });
                        return Ok(Obj::NONE);
                    }

                    if result.obj.try_as_obj::<Task>().is_some() {
                        self.routine_wait.set(RoutineWait::Task(result.obj));
                        return Ok(Obj::NONE);
                    }

                    // Unknown wake signals already cause the event loop to requeue the outer task.
                    return Ok(result.obj);
                }
                VmReturnKind::Normal => {
                    self.clear_phase_routine();
                    match self.phase.get() {
                        Phase::Connected | Phase::Disconnected => {
                            self.enter_current_mode()?;
                        }
                        // A completed mode routine remains stopped until the mode changes.
                        Phase::Mode(_) => return Ok(Obj::NONE),
                        Phase::Initial => unreachable!(),
                    }
                }
                VmReturnKind::Exception => nlr::raise(token(), result.obj),
            }
        }
    }
}

#[class_methods]
impl Competition {
    #[make_new]
    #[stub(sig = "(self) -> None")]
    fn make_new(ty: &'static ObjType, _n_pos: usize, _n_kw: usize, args: &[Obj]) -> Self {
        if !args.is_empty() {
            type_error(c"function does not accept arguments").raise(token());
        }

        Self {
            base: ObjBase::new(ty),

            connected: Cell::new(None),
            disconnected: Cell::new(None),
            driver: Cell::new(None),
            autonomous: Cell::new(None),
            disabled: Cell::new(None),
        }
    }

    #[method]
    #[stub(sig = "(self, routine: Callable[..., Any]) -> Callable[..., Any]")]
    fn connected(&self, routine: Callable) -> Obj {
        self.connected.set(Some(routine));
        routine.into_inner()
    }

    #[method]
    #[stub(sig = "(self, routine: Callable[..., Any]) -> Callable[..., Any]")]
    fn disconnected(&self, routine: Callable) -> Obj {
        self.disconnected.set(Some(routine));
        routine.into_inner()
    }

    #[method]
    #[stub(sig = "(self, routine: Callable[..., Any]) -> Callable[..., Any]")]
    fn driver(&self, routine: Callable) -> Obj {
        self.driver.set(Some(routine));
        routine.into_inner()
    }

    #[method]
    #[stub(sig = "(self, routine: Callable[..., Any]) -> Callable[..., Any]")]
    fn autonomous(&self, routine: Callable) -> Obj {
        self.autonomous.set(Some(routine));
        routine.into_inner()
    }

    #[method]
    #[stub(sig = "(self, routine: Callable[..., Any]) -> Callable[..., Any]")]
    fn disabled(&self, routine: Callable) -> Obj {
        self.disabled.set(Some(routine));
        routine.into_inner()
    }

    #[method]
    fn run(&self) -> CompetitionRuntime {
        CompetitionRuntime {
            base: ObjBase::new(CompetitionRuntime::OBJ_TYPE),

            status: Cell::new(status()),
            phase: Cell::new(Phase::Initial),

            connected: self.connected.get(),
            disconnected: self.disconnected.get(),
            driver: self.driver.get(),
            autonomous: self.autonomous.get(),
            disabled: self.disabled.get(),

            coro: Cell::new(Obj::NULL),
            routine_wait: Cell::new(RoutineWait::Ready),
        }
    }
}

#[class_methods]
impl CompetitionRuntime {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
        self_in.as_obj::<Self>().tick().into()
    }
}
