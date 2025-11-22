use micropython_rs::obj::{Obj, ObjTrait};

use crate::GC;

pub fn alloc_obj<T: ObjTrait + 'static>(o: T) -> Obj {
    let gc = GC.gc().unwrap();
    let mut lock = gc.lock().unwrap();
    Obj::new(o, &mut lock, false).unwrap()
}

pub fn alloc_obj_with_finaliser<T: ObjTrait + 'static>(o: T) -> Obj {
    let gc = GC.gc().unwrap();
    let mut lock = gc.lock().unwrap();
    Obj::new(o, &mut lock, true).unwrap()
}
