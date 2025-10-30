use std::cell::{Ref, RefCell};

use cty::c_void;
use micropython_rs::{
    except::raise_type_error,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};

use crate::qstrgen::qstr;

static TASK_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(Task))
    .set_slot_iter(task_iternext as *const c_void);

#[repr(C)]
pub struct Task {
    base: ObjBase,
    // generator object
    coro: Obj,
    waiting_tasks: RefCell<Vec<Obj>>,
}

unsafe impl ObjTrait for Task {
    const OBJ_TYPE: *const micropython_rs::obj::ObjType = TASK_OBJ_TYPE.as_obj_type_ptr();
}

impl Task {
    pub fn new(coro: Obj) -> Self {
        Self {
            base: ObjBase::new::<Self>(),
            coro,
            waiting_tasks: RefCell::new(Vec::new()),
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
}

extern "C" fn task_iternext(self_in: Obj) -> Obj {
    let task = self_in
        .as_obj::<Task>()
        .unwrap_or_else(|| raise_type_error(token().unwrap(), "expected Task"));
    task.add_waiting_task(todo!());
}
