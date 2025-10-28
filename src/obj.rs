use micropython_rs::obj::{Obj, ObjTrait};

use crate::ALLOCATOR;

pub fn alloc_obj<T: ObjTrait>(o: T) -> Obj {
    Obj::new(o, ALLOCATOR.lock().as_mut().unwrap()).unwrap()
}
