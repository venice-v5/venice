use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::MicroPython;

pub static REENTRANCE_ALLOWED: AtomicBool = AtomicBool::new(false);

impl MicroPython {
    pub fn allow_reentrance<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let old = REENTRANCE_ALLOWED.swap(true, Ordering::Relaxed);
        let ret = f();
        REENTRANCE_ALLOWED.store(old, Ordering::Relaxed);
        ret
    }

    pub fn reenter<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        match REENTRANCE_ALLOWED.compare_exchange(true, false, Ordering::Release, Ordering::Acquire)
        {
            Ok(_) => {
                let mut this = ManuallyDrop::new(Self(()));
                let ret = f(&mut this);
                REENTRANCE_ALLOWED.store(true, Ordering::Release);
                ret
            }
            Err(_) => panic!("reetrance attempted while prohibited"),
        }
    }
}
