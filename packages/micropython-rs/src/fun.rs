use crate::{
    map::Map,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};

mod sealed {
    pub trait Sealed {}
}

pub trait Fun: sealed::Sealed + ObjTrait {}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FunSig(u32);

impl FunSig {
    pub const fn new(args_min: u16, args_max: u16, takes_kw: bool) -> Self {
        let sig = (args_min as u32) << 17 | (args_max as u32) << 1 | (takes_kw as u32);
        Self(sig)
    }
}

macro_rules! define_fixed_fun_type {
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

        impl sealed::Sealed for $name {}
        impl Fun for $name {}

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

unsafe extern "C" {
    static mp_type_fun_builtin_var: ObjType;
}

define_fixed_fun_type!(Fun0, extern "C" fn() -> Obj, mp_type_fun_builtin_0);
define_fixed_fun_type!(Fun1, extern "C" fn(Obj) -> Obj, mp_type_fun_builtin_1);
define_fixed_fun_type!(Fun2, extern "C" fn(Obj, Obj) -> Obj, mp_type_fun_builtin_2);
define_fixed_fun_type!(
    Fun3,
    extern "C" fn(Obj, Obj, Obj) -> Obj,
    mp_type_fun_builtin_3
);

macro_rules! define_var_fun_type {
    ($name:ident, $ty:ty) => {
        #[repr(C)]
        pub struct $name {
            base: ObjBase<'static>,
            sig: FunSig,
            fun: $ty,
        }

        impl sealed::Sealed for $name {}
        impl Fun for $name {}

        unsafe impl ObjTrait for $name {
            const OBJ_TYPE: &ObjType = unsafe { &mp_type_fun_builtin_var };
        }
    };
}

define_var_fun_type!(
    FunVar,
    unsafe extern "C" fn(n: usize, ptr: *const Obj) -> Obj
);
define_var_fun_type!(
    FunVarBetween,
    unsafe extern "C" fn(n: usize, ptr: *const Obj) -> Obj
);
define_var_fun_type!(
    FunVarKw,
    unsafe extern "C" fn(n: usize, ptr: *const Obj, map: *mut Map) -> Obj
);

pub const ARGS_MAX: u16 = u16::MAX;

impl FunVar {
    pub const fn new(
        f: unsafe extern "C" fn(n: usize, ptr: *const Obj) -> Obj,
        args_min: u16,
    ) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            sig: FunSig::new(args_min, ARGS_MAX, false),
            fun: f,
        }
    }
}

impl FunVarBetween {
    pub const fn new(
        f: unsafe extern "C" fn(n: usize, ptr: *const Obj) -> Obj,
        args_min: u16,
        args_max: u16,
    ) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            sig: FunSig::new(args_min, args_max, false),
            fun: f,
        }
    }
}

impl FunVarKw {
    pub const fn new(
        f: unsafe extern "C" fn(n: usize, ptr: *const Obj, map: *mut Map) -> Obj,
        args_min: u16,
    ) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            sig: FunSig::new(args_min, ARGS_MAX, true),
            fun: f,
        }
    }
}

macro_rules! define_static_method_type {
    ($name:ident, $mp_ty:ident) => {
        #[repr(C)]
        pub struct $name {
            base: ObjBase<'static>,
            f_obj: Obj,
        }

        unsafe extern "C" {
            static $mp_ty: ObjType;
        }

        unsafe impl ObjTrait for $name {
            const OBJ_TYPE: &ObjType = unsafe { &$mp_ty };
        }

        impl $name {
            pub const fn new<F: Fun>(f: &'static F) -> Self {
                Self {
                    base: ObjBase::new(Self::OBJ_TYPE),
                    f_obj: Obj::from_static(f),
                }
            }
        }
    };
}

define_static_method_type!(StaticMethod, mp_type_staticmethod);
define_static_method_type!(ClassMethod, mp_type_classmethod);
