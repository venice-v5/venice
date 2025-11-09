use crate::obj::{Obj, ObjBase, ObjTrait, ObjType};

macro_rules! define_fun_type {
    ($name:ident, $fn_type:ty, $mp_type_name:ident) => {
        #[repr(C)]
        pub struct $name {
            base: ObjBase<'static>,
            fun: $fn_type,
        }

        unsafe extern "C" {
            static $mp_type_name: ObjType;
        }

        unsafe impl ObjTrait for $name {
            const OBJ_TYPE: &ObjType = unsafe { &$mp_type_name };
        }

        impl $name {
            pub const fn new(f: $fn_type) -> Self {
                Self {
                    base: ObjBase::new(Self::OBJ_TYPE),
                    fun: f,
                }
            }
        }
    };
}

define_fun_type!(Fun0, extern "C" fn() -> Obj, mp_type_fun_builtin_0);
define_fun_type!(Fun1, extern "C" fn(Obj) -> Obj, mp_type_fun_builtin_1);
define_fun_type!(Fun2, extern "C" fn(Obj, Obj) -> Obj, mp_type_fun_builtin_2);
define_fun_type!(
    Fun3,
    extern "C" fn(Obj, Obj, Obj) -> Obj,
    mp_type_fun_builtin_3
);
define_fun_type!(
    FunVar,
    unsafe extern "C" fn(n_args: usize, ptr: *const Obj) -> Obj,
    mp_type_fun_builtin_var
);
