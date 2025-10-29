use micropython_rs::obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags};

use crate::qstrgen::qstr;

static TASK_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Task));

#[repr(C)]
pub struct Task {
    base: ObjBase,
    // generator object
    coro: Obj,
}

unsafe impl ObjTrait for Task {
    const OBJ_TYPE: *const micropython_rs::obj::ObjType =
        &raw const TASK_OBJ_TYPE as *const ObjType;
}

impl Task {
    pub fn new(coro: Obj) -> Self {
        Self {
            base: ObjBase::new::<Self>(),
            coro,
        }
    }

    pub fn coro(&self) -> Obj {
        self.coro
    }
}
