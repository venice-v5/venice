use crate::obj::Obj;

unsafe extern "C" {
    fn mp_obj_new_tuple(n: usize, items: *const Obj) -> Obj;
}

pub fn new_tuple(items: &[Obj]) -> Obj {
    unsafe { mp_obj_new_tuple(items.len(), items.as_ptr()) }
}
