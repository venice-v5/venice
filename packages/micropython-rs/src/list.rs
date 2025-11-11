use crate::obj::Obj;

unsafe extern "C" {
    fn mp_obj_new_list(n: usize, items: *const Obj) -> Obj;
}

impl Obj {
    pub fn from_list(items: Vec<Obj>) -> Self {
        unsafe { mp_obj_new_list(items.len(), items.as_ptr()) }
    }
}
