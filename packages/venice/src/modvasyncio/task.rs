use std::cell::{Cell, Ref, RefCell};

use micropython_rs::{
    except::{raise_stop_iteration, raise_type_error},
    init::token,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};

use crate::{alloc::Gc, qstrgen::qstr};

static TASK_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(Task))
    .set_iter(Iter::IterNext(task_iternext));

#[repr(C)]
pub struct Task {
    base: ObjBase<'static>,
    // generator object
    coro: Obj,
    waiting_tasks: RefCell<Vec<Obj, Gc>>,
    return_val: Cell<Obj>,
}

unsafe impl ObjTrait for Task {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = TASK_OBJ_TYPE.as_obj_type();
}

impl Task {
    pub fn new(coro: Obj) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
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

extern "C" fn task_iternext(self_in: Obj) -> Obj {
    let task = self_in
        .try_as_obj::<Task>()
        .unwrap_or_else(|| raise_type_error(token(), c"expected Task"));
    if !task.is_complete() {
        self_in
    } else {
        raise_stop_iteration(token(), task.return_val.get())
    }
}
