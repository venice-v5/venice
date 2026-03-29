use std::cell::{Cell, Ref, RefCell};

use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{raise_stop_iteration, type_error},
    init::token,
    obj::{Obj, ObjBase, ObjTrait},
};

use crate::alloc::Gc;

#[class(qstr!(Task))]
#[repr(C)]
pub struct Task {
    base: ObjBase,
    // generator object
    coro: Obj,
    waiting_tasks: RefCell<Vec<Obj, Gc>>,
    return_val: Cell<Obj>,
}

impl Task {
    pub fn new(coro: Obj) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            coro,
            waiting_tasks: RefCell::new(Vec::new_in(Gc { token: token() })),
            return_val: Cell::new(Obj::NULL),
        }
    }

    pub fn coro(&self) -> Obj {
        self.coro
    }

    pub fn add_waiting_task(&self, task: Obj) {
        self.waiting_tasks.borrow_mut().push(task);
    }

    pub fn waiting_tasks<'a>(&'a self) -> Ref<'a, [Obj]> {
        Ref::map(self.waiting_tasks.borrow(), |tasks| tasks.as_slice())
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
        let task = self_in
            .try_as_obj::<Task>()
            .unwrap_or_else(|| type_error(c"expected Task").raise(token()));
        if !task.is_complete() {
            self_in
        } else {
            raise_stop_iteration(token(), task.return_val.get())
        }
    }
}
