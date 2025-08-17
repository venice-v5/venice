use crate::obj::{Obj, ObjBase};

pub type Fun0 = extern "C" fn();
pub type Fun1 = extern "C" fn(a: Obj);
pub type Fun2 = extern "C" fn(a: Obj, b: Obj);
pub type Fun3 = extern "C" fn(a: Obj, b: Obj, c: Obj);

mod sealed {
    use super::{Fun0, Fun1, Fun2, Fun3};
    use crate::obj::ObjFullType;

    pub trait FunType {
        const TYPE_OBJ: *const ObjFullType;
    }

    unsafe extern "C" {
        static mp_type_fun_builtin_0: ObjFullType;
        static mp_type_fun_builtin_1: ObjFullType;
        static mp_type_fun_builtin_2: ObjFullType;
        static mp_type_fun_builtin_3: ObjFullType;
    }

    impl FunType for Fun0 {
        const TYPE_OBJ: *const ObjFullType = &raw const mp_type_fun_builtin_0;
    }

    impl FunType for Fun1 {
        const TYPE_OBJ: *const ObjFullType = &raw const mp_type_fun_builtin_1;
    }

    impl FunType for Fun2 {
        const TYPE_OBJ: *const ObjFullType = &raw const mp_type_fun_builtin_2;
    }

    impl FunType for Fun3 {
        const TYPE_OBJ: *const ObjFullType = &raw const mp_type_fun_builtin_3;
    }
}

use sealed::FunType;

#[repr(C)]
pub struct Fun<F: FunType> {
    base: ObjBase,
    fun: F,
}

impl<F: FunType> Fun<F> {
    pub const fn new(f: F) -> Self {
        Self {
            base: ObjBase::new(unsafe { &*F::TYPE_OBJ }),
            fun: f,
        }
    }
}
