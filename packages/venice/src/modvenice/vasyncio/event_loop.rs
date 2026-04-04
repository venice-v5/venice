use std::{
    cell::{Cell, RefCell},
    collections::{binary_heap::BinaryHeap, vec_deque::VecDeque},
};

use micropython_macros::{class, class_methods, fun};
use micropython_rs::{
    except::{RUNTIME_ERROR_TYPE, raise_msg, type_error},
    fun::{Fun1, Fun2},
    generator::{GEN_INSTANCE_TYPE, VmReturnKind, resume_gen},
    init::token,
    nlr::{self, push_nlr_callback},
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};
use vex_sdk::vexTasksRun;

use super::{sleep::Sleep, task::Task, time32};
use crate::{alloc::Gc, modvenice::Exception, obj::alloc_obj};

struct Sleeper {
    task: Obj,
    deadline: time32::Instant,
    sleep: Obj,
}

impl PartialEq for Sleeper {
    fn eq(&self, other: &Self) -> bool {
        self.deadline.eq(&other.deadline)
    }
}

impl Eq for Sleeper {}

impl PartialOrd for Sleeper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sleeper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deadline.cmp(&other.deadline).reverse()
    }
}

#[class(qstr!(WakeSignal))]
#[repr(C)]
pub struct WakeSignal {
    base: ObjBase,
}

#[class_methods]
impl WakeSignal {}

pub static WAKE_SIGNAL: WakeSignal = WakeSignal {
    base: ObjBase::new(WakeSignal::OBJ_TYPE),
};

#[class(qstr!(EventLoop))]
#[repr(C)]
pub struct EventLoop {
    base: ObjBase,
    ready: RefCell<VecDeque<Obj, Gc>>,
    sleepers: RefCell<BinaryHeap<Sleeper, Gc>>,
    current_task: Cell<Obj>,
}

thread_local! {
    static RUNNING_LOOP: Cell<Obj> = const { Cell::new(Obj::NONE) };
}

impl EventLoop {
    pub fn new() -> Self {
        let gc = Gc { token: token() };
        Self {
            base: Self::OBJ_TYPE.into(),
            ready: RefCell::new(VecDeque::new_in(gc)),
            sleepers: RefCell::new(BinaryHeap::new_in(gc)),
            current_task: Cell::new(Obj::NULL),
        }
    }

    pub fn spawn(&self, coro: Obj) -> Obj {
        let task = alloc_obj(Task::new(coro));
        self.ready.borrow_mut().push_back(task);
        task
    }

    /// `task_obj` and `coro` are nullable
    pub fn tick_coro(&self, mut task_obj: Obj, mut coro: Obj) -> bool {
        if task_obj.is_null() {
            task_obj = self.current_task.get()
        }

        let task = task_obj.try_as_obj::<Task>().unwrap();
        if coro.is_null() {
            coro = task.coro();
        }
        assert!(coro.is(micropython_rs::generator::GEN_INSTANCE_TYPE));

        let prev_task_obj = self.current_task.replace(task_obj);
        let result = resume_gen(coro, Obj::NONE, Obj::NULL);
        let terminated = match result.return_kind {
            VmReturnKind::Normal => {
                let mut ready = self.ready.borrow_mut();
                task.complete_with(result.obj);
                task.waiting_tasks()
                    .iter()
                    .for_each(|&w| ready.push_front(w));

                true
            }
            VmReturnKind::Yield => {
                if let Some(sleep) = result.obj.try_as_obj::<Sleep>() {
                    self.sleepers.borrow_mut().push(Sleeper {
                        task: task_obj,
                        deadline: time32::Instant::now() + sleep.duration(),
                        sleep: result.obj,
                    });
                } else if let Some(awaited_task) = result.obj.try_as_obj::<Task>() {
                    awaited_task.add_waiting_task(task_obj);
                } else if result.obj.is(WakeSignal::OBJ_TYPE) {
                    self.ready.borrow_mut().push_back(task_obj);
                }

                false
            }
            VmReturnKind::Exception => nlr::raise(token(), result.obj),
        };

        self.current_task.set(prev_task_obj);
        terminated
    }

    // returns:
    // true -> no more tasks/sleepers to run, stop
    // false -> tasks/sleepers still in queues
    pub fn tick(&self) -> bool {
        let mut ready = self.ready.borrow_mut();
        let mut sleepers = self.sleepers.borrow_mut();

        if let Some(sleeper) = sleepers.peek()
            && sleeper.deadline <= super::time32::Instant::now()
        {
            let sleeper = sleepers.pop().unwrap();
            sleeper.sleep.try_as_obj::<Sleep>().unwrap().complete();
            ready.push_back(sleeper.task);
        }

        let task_obj = ready.pop_front();
        // let the task access the event loop while it's running
        drop(ready);
        drop(sleepers);

        if let Some(task_obj) = task_obj {
            self.tick_coro(task_obj, Obj::NULL);
        }

        unsafe { vexTasksRun() };
        self.sleepers.borrow().is_empty() && self.ready.borrow().is_empty()
    }

    pub fn run(&self) {
        while !self.tick() {}
    }
}

#[class_methods]
impl EventLoop {
    #[make_new]
    fn make_new(
        _: &ObjType,
        _n_args: usize,
        _n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        if args.len() != 0 {
            Err(type_error(
                c"constructor does not accept arguments; just call EventLoop()",
            ))?
        }

        Ok(Self::new())
    }

    // this function can't use a Fun generator because a Generator struct would be needed to write out
    // its type signature, and that struct does not exist
    extern "C" fn py_spawn(self_in: Obj, coro: Obj) -> Obj {
        if !coro.is(GEN_INSTANCE_TYPE) {
            type_error(c"expected coroutine").raise(token());
        }

        self_in.try_as_obj::<EventLoop>().unwrap().spawn(coro)
    }

    #[constant(qstr!(spawn))]
    const SPAWN: &Fun2 = &Fun2::new(Self::py_spawn);

    // this function can't use a Fun generator because it needs the EventLoop in Obj form, not as a
    // reference, in order to properly replace the static variable
    extern "C" fn py_run(self_in: Obj) -> Obj {
        let prev_loop = RUNNING_LOOP.replace(self_in);
        push_nlr_callback(
            token(),
            || self_in.try_as_obj::<EventLoop>().unwrap().run(),
            || RUNNING_LOOP.set(prev_loop),
            true,
        );
        Obj::NONE
    }

    #[constant(qstr!(run))]
    const RUN: &Fun1 = &Fun1::new(Self::py_run);
}

#[fun]
pub fn run(coro: Obj) -> Obj {
    if !coro.is(GEN_INSTANCE_TYPE) {
        type_error(c"expected coroutine").raise(token());
    }

    let eloop = EventLoop::new();
    eloop.spawn(coro);
    EventLoop::py_run(alloc_obj(eloop))
}

#[fun]
pub fn spawn(coro: Obj) -> Obj {
    let eloop = RUNNING_LOOP.get();
    if eloop.is_none() {
        raise_msg(token(), RUNTIME_ERROR_TYPE, c"no running event loop");
    }

    EventLoop::py_spawn(eloop, coro)
}

#[fun]
pub fn get_running_loop() -> Obj {
    RUNNING_LOOP.get()
}
