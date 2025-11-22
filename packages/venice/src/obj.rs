use micropython_rs::{
    init::token,
    obj::{Obj, ObjTrait},
};

pub fn alloc_obj<T: ObjTrait + 'static>(o: T) -> Obj {
    Obj::new(token().unwrap(), o, false).unwrap()
}

pub fn alloc_obj_with_finaliser<T: ObjTrait + 'static>(o: T) -> Obj {
    Obj::new(token().unwrap(), o, true).unwrap()
}
