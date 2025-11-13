use crate::obj::Obj;

unsafe extern "C" {
    fn mp_obj_new_list(n: usize, items: *const Obj) -> Obj;
}

pub fn new_list(items: &[Obj]) -> Obj {
    unsafe { mp_obj_new_list(items.len(), items.as_ptr()) }
}
