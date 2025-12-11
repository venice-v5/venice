use micropython_rs::{
    init::token,
    obj::{Obj, ObjTrait},
};

pub fn alloc_obj<T: ObjTrait + 'static>(o: T) -> Obj {
    Obj::new(token().unwrap(), o, false).unwrap()
}
