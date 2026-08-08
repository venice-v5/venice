use std::cell::{Cell, RefCell};

use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::raise_stop_iteration,
    init::token,
    obj::{Obj, ObjBase, ObjTrait},
};

use crate::alloc::Gc;

/// A spawned task.
///
/// A `Task` can be awaited to retrieve the output of its coroutine.
///
/// `EventLoop.spawn` and `vasyncio.spawn` return tasks; `Task` is not directly exported from the
/// `vasyncio` submodule and is not constructed by users. Awaiting a task cooperatively waits for
/// its coroutine and returns that coroutine's return value, including when the task completed
/// before the await began. Direct or transitive cycles between awaited tasks raise `RuntimeError`.
/// A coroutine exception propagates out of the running event loop. `Task` objects cannot be
/// cancelled.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// async def work():
///     print("Hello from a task!")
///     return 1 + 2
///
/// async def main():
///     # Spawn a coroutine onto the event loop.
///     task = vasyncio.spawn(work())
///
///     # Wait for the task's output.
///     assert await task == 3
///
/// vasyncio.run(main())
/// ```
#[class(qstr!(Task))]
#[repr(C)]
pub struct Task {
    base: ObjBase,
    // generator object
    coro: Obj,
    waiting_tasks: RefCell<Vec<Obj, Gc>>,
    waiting_on: Cell<Obj>,
    return_val: Cell<Obj>,
}

impl Task {
    pub fn new(coro: Obj) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            coro,
            waiting_tasks: RefCell::new(Vec::new_in(Gc { token: token() })),
            waiting_on: Cell::new(Obj::NULL),
            return_val: Cell::new(Obj::NULL),
        }
    }

    pub fn coro(&self) -> Obj {
        self.coro
    }

    pub fn add_waiting_task(&self, task: Obj) {
        self.waiting_tasks.borrow_mut().push(task);
    }

    pub fn pop_waiting_task(&self) -> Option<Obj> {
        self.waiting_tasks.borrow_mut().pop()
    }

    pub fn waiting_on(&self) -> Obj {
        self.waiting_on.get()
    }

    pub fn set_waiting_on(&self, task: Obj) {
        self.waiting_on.set(task);
    }

    pub fn clear_waiting_on(&self) {
        self.waiting_on.set(Obj::NULL);
    }

    pub fn is_complete(&self) -> bool {
        !self.return_val.get().is_null()
    }

    pub fn complete_with(&self, val: Obj) {
        self.return_val.set(val);
    }
}

#[class_methods]
impl Task {
    #[iter]
    extern "C" fn task_iternext(self_in: Obj) -> Obj {
        let task = self_in.as_obj::<Task>();
        if !task.is_complete() {
            self_in
        } else {
            raise_stop_iteration(token(), task.return_val.get())
        }
    }
}
