use std::{
    cell::{Cell, RefCell},
    collections::{binary_heap::BinaryHeap, vec_deque::VecDeque},
};

use micropython_macros::{class, class_methods, fun};
use micropython_rs::{
    except::{RUNTIME_ERROR_TYPE, raise_msg, runtime_error, type_error, value_error},
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

/// A cooperative scheduler for Venice coroutine tasks and timed sleeps.
///
/// A *task* is a lightweight, non-blocking unit of execution. Tasks allow you to cooperatively perform
/// work in the background without blocking other code from running.
///
/// - Tasks are **lightweight**. Because tasks are scheduled and managed by Venice, creating new tasks
///   or switching between tasks does not require a context switch and has fairly low overhead.
///   Creating, running, and destroying large numbers of tasks is relatively cheap in comparison to
///   traditional threads.
/// - Tasks are scheduled **cooperatively**. A task will run until it voluntarily yields using an
///   `await` point, giving control back to the event loop. The loop then switches to executing a
///   different task.
/// - Tasks are **non-blocking**. When a task cannot continue executing, it should yield, allowing the
///   event loop to schedule another task in its place. Tasks should not perform operations that could
///   block the CPU for a long period without an `await` point, because this prevents other tasks from
///   executing as well. This includes long-running tight loops without `await` points.
///
/// The loop runs one ready task at a time. Users normally call `vasyncio.run` instead of managing an
/// event loop directly.
#[class(qstr!(EventLoop))]
#[repr(C)]
pub struct EventLoop {
    base: ObjBase,
    ready: RefCell<VecDeque<Obj, Gc>>,
    sleepers: RefCell<BinaryHeap<Sleeper, Gc>>,
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
        }
    }

    pub fn spawn(&self, coro: Obj) -> Obj {
        let task = alloc_obj(Task::new(coro));
        self.ready.borrow_mut().push_back(task);
        task
    }

    fn await_would_cycle(waiting_task: Obj, mut awaited_task: Obj) -> bool {
        loop {
            if waiting_task.inner() == awaited_task.inner() {
                return true;
            }

            let dependency = awaited_task.as_obj::<Task>().waiting_on();
            if dependency.is_null() {
                return false;
            }
            awaited_task = dependency;
        }
    }

    /// Resumes a task's root coroutine and schedules the task from the yielded result.
    ///
    /// Child coroutines and custom awaitables must delegate their yielded objects to this root coroutine.
    /// Scheduling a child through this method would incorrectly treat child completion as task
    /// completion.
    fn tick_task(&self, task_obj: Obj) {
        let task = task_obj.as_obj::<Task>();
        let coro = task.coro();
        assert!(coro.is(GEN_INSTANCE_TYPE));

        let result = resume_gen(coro, Obj::NONE, Obj::NULL);
        match result.return_kind {
            VmReturnKind::Normal => {
                let mut ready = self.ready.borrow_mut();
                task.complete_with(result.obj);
                while let Some(waiting) = task.pop_waiting_task() {
                    waiting.as_obj::<Task>().clear_waiting_on();
                    ready.push_front(waiting);
                }
            }
            VmReturnKind::Yield => {
                if let Some(sleep) = result.obj.try_as_obj::<Sleep>() {
                    let deadline = time32::Instant::now()
                        .checked_add(sleep.duration())
                        .unwrap_or_else(|| {
                            value_error(c"sleep deadline is too large").raise(token())
                        });
                    self.sleepers.borrow_mut().push(Sleeper {
                        task: task_obj,
                        deadline,
                        sleep: result.obj,
                    });
                } else if let Some(awaited_task) = result.obj.try_as_obj::<Task>() {
                    if awaited_task.is_complete() {
                        self.ready.borrow_mut().push_front(task_obj);
                    } else {
                        if Self::await_would_cycle(task_obj, result.obj) {
                            runtime_error(c"task await cycle detected").raise(token());
                        }
                        task.set_waiting_on(result.obj);
                        awaited_task.add_waiting_task(task_obj);
                    }
                } else {
                    self.ready.borrow_mut().push_back(task_obj);
                }
            }
            VmReturnKind::Exception => nlr::raise(token(), result.obj),
        }
    }

    // returns:
    // true -> no more tasks/sleepers to run, stop
    // false -> tasks/sleepers still in queues
    pub fn tick(&self) -> bool {
        let mut ready = self.ready.borrow_mut();
        let mut sleepers = self.sleepers.borrow_mut();

        let now = super::time32::Instant::now();
        while let Some(sleeper) = sleepers.peek()
            && sleeper.deadline <= now
        {
            let sleeper = sleepers.pop().unwrap();
            sleeper.sleep.as_obj::<Sleep>().complete();
            ready.push_back(sleeper.task);
        }

        let task_obj = ready.pop_front();
        // let the task access the event loop while it's running
        drop(ready);
        drop(sleepers);

        if let Some(task_obj) = task_obj {
            self.tick_task(task_obj);
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
    /// Creates an empty event loop.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If any positional or keyword arguments are supplied.
    #[make_new]
    #[stub(sig = "(self, /) -> None")]
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

    // TODO: refactor these functions, they can probably be expressed by the new function generators

    // this function can't use a Fun generator because a Generator struct would be needed to write out
    // its type signature, and that struct does not exist
    extern "C" fn py_spawn(self_in: Obj, coro: Obj) -> Obj {
        if !coro.is(GEN_INSTANCE_TYPE) {
            type_error(c"expected coroutine").raise(token());
        }

        self_in.as_obj::<EventLoop>().spawn(coro)
    }

    /// Schedules coroutine object `coro` on this loop and returns an awaitable `Task`.
    ///
    /// Scheduling does not run the coroutine until the loop is running. Awaiting the returned task
    /// yields its return value whether the task is still running or has already finished.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `coro` is not a coroutine object.
    #[constant(qstr!(spawn))]
    #[stub(sig = "(self, coro: Any, /) -> Task")]
    const SPAWN: &Fun2 = &Fun2::new(Self::py_spawn);

    // this function can't use a Fun generator because it needs the EventLoop in Obj form, not as a
    // reference, in order to properly replace the static variable
    extern "C" fn py_run(self_in: Obj) -> Obj {
        let prev_loop = RUNNING_LOOP.replace(self_in);
        push_nlr_callback(
            token(),
            || self_in.as_obj::<EventLoop>().run(),
            || RUNNING_LOOP.set(prev_loop),
            true,
        );
        Obj::NONE
    }

    /// Runs scheduled tasks until no ready tasks or pending sleeps remain.
    ///
    /// While this method is running, `vasyncio.get_running_loop` returns this loop and
    /// `vasyncio.spawn` adds tasks to it. An exception raised by a task stops the loop and is
    /// propagated to the caller.
    ///
    /// # Raises
    ///
    /// - `RuntimeError`: If tasks form a direct or transitive await cycle.
    /// - `ValueError`: If a `Sleep` deadline is too large to represent.
    #[constant(qstr!(run))]
    #[stub(sig = "(self, /) -> None")]
    const RUN: &Fun1 = &Fun1::new(Self::py_run);
}

/// Runs coroutine object `coro` on a new event loop until no work remains.
///
/// The loop also waits for tasks spawned into it and pending `Sleep` objects before returning
/// `None`. The root coroutine's return value is discarded, and an exception from any task stops
/// the loop and is propagated to the caller.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// async def main():
///     print("start")
///     await vasyncio.Sleep(100, MILLIS)
///     print("done")
///
/// vasyncio.run(main())
/// ```
///
/// # Raises
///
/// - `TypeError`: If `coro` is not a coroutine object.
/// - `RuntimeError`: If tasks form a direct or transitive await cycle.
/// - `ValueError`: If a `Sleep` deadline is too large to represent.
#[fun]
#[stub(sig = "(coro: Any, /) -> None")]
pub fn run(coro: Obj) -> Obj {
    if !coro.is(GEN_INSTANCE_TYPE) {
        type_error(c"expected coroutine").raise(token());
    }

    let eloop = EventLoop::new();
    eloop.spawn(coro);
    EventLoop::py_run(alloc_obj(eloop))
}

/// Spawns a new asynchronous task that can be controlled with the returned `Task` handle.
///
/// Call this from code already executing under `vasyncio.run` or `EventLoop.run`. Awaiting the task
/// yields the coroutine's return value whether the task is still running or has already completed.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// async def answer():
///     await vasyncio.Sleep(10, MILLIS)
///     return 42
///
/// async def main():
///     task = vasyncio.spawn(answer())
///     print(await task)
///
/// vasyncio.run(main())
/// ```
///
/// # Raises
///
/// - `RuntimeError`: If no event loop is running.
/// - `TypeError`: If `coro` is not a coroutine object.
#[fun]
#[stub(sig = "(coro: Any, /) -> Task")]
pub fn spawn(coro: Obj) -> Obj {
    let eloop = RUNNING_LOOP.get();
    if eloop.is_none() {
        raise_msg(token(), RUNTIME_ERROR_TYPE, c"no running event loop");
    }

    EventLoop::py_spawn(eloop, coro)
}

/// Returns the event loop currently executing tasks, or `None` outside `vasyncio.run` or
/// `EventLoop.run`.
#[fun]
#[stub(sig = "() -> EventLoop | None")]
pub fn get_running_loop() -> Obj {
    RUNNING_LOOP.get()
}
