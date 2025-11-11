use crate::obj::Obj;

unsafe extern "C" {
    fn mp_obj_new_tuple(n: usize, items: *const Obj) -> Obj;
}

impl Obj {
    pub fn from_tuple(items: Vec<Obj>) -> Self {
        unsafe { mp_obj_new_tuple(items.len(), items.as_ptr()) }
    }
}
