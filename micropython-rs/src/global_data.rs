use core::cell::UnsafeCell;

use hashbrown::HashMap;

use crate::MicroPython;

// A mutex would not work here. We want the `&GlobalData` returned by `MicroPython` to be tied to
// it and have its lifetime.
static GLOBAL_DATA: GdContainer = GdContainer {
    inner: UnsafeCell::new(None),
};

struct GdContainer {
    inner: UnsafeCell<Option<GlobalData>>,
}

unsafe impl Sync for GdContainer {}

pub struct GlobalData {
    pub module_map: HashMap<&'static [u8], &'static [u8]>,
}

impl MicroPython {
    pub fn global_data(&self) -> &GlobalData {
        // SAFETY: There will only ever be one `MicroPython` in existence
        unsafe { &*GLOBAL_DATA.inner.get() }.as_ref().unwrap()
    }

    pub(crate) unsafe fn set_global_data(&mut self, gd: GlobalData) {
        unsafe { &mut *GLOBAL_DATA.inner.get() }.replace(gd);
    }
}
