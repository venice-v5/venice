use crate::{init::InitToken, obj::Obj};

unsafe extern "C" {
    fn mp_raise_StopIteration(arg: Obj) -> !;
}

pub fn raise_stop_iteration(_: InitToken, arg: Obj) -> ! {
    unsafe {
        mp_raise_StopIteration(arg);
    }
}
