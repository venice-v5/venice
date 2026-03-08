macro_rules! fun1 {
    ($f:expr, $a:ty) => {{
        use ::argparse::{ArgTrait, ArgValue};
        use ::micropython_rs::{except::raise_type_error, fun::Fun1, init::token, obj::Obj};

        extern "C" fn trampoline(a: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);

            if !<$a as ArgTrait>::coercable(a_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #1, found <{}>",
                        <$a as ArgTrait>::ty(),
                        a_value.ty()
                    ),
                );
            }

            unsafe { $f(<$a as ArgTrait>::from_arg_value(a_value).unwrap_unchecked()) }
        }

        Fun1::new(trampoline)
    }};
}

macro_rules! fun2 {
    ($f:expr, $a:ty, $b:ty) => {{
        use ::argparse::{ArgTrait, ArgValue};
        use ::micropython_rs::{except::raise_type_error, fun::Fun2, init::token, obj::Obj};

        extern "C" fn trampoline(a: Obj, b: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);
            let b_value = ArgValue::from_obj(&b);

            if !<$a as ArgTrait>::coercable(a_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #1, found <{}>",
                        <$a as ArgTrait>::ty(),
                        a_value.ty()
                    ),
                );
            }

            if !<$b as ArgTrait>::coercable(b_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #2, found <{}>",
                        <$b as ArgTrait>::ty(),
                        b_value.ty()
                    ),
                );
            }

            unsafe {
                $f(
                    <$a as ArgTrait>::from_arg_value(a_value).unwrap_unchecked(),
                    <$b as ArgTrait>::from_arg_value(b_value).unwrap_unchecked(),
                )
            }
        }

        Fun2::new(trampoline)
    }};
}

macro_rules! fun3 {
    ($f:expr, $a:ty, $b:ty, $c:ty) => {{
        use ::argparse::{ArgTrait, ArgValue};
        use ::micropython_rs::{except::raise_type_error, fun::Fun3, init::token, obj::Obj};

        extern "C" fn trampoline(a: Obj, b: Obj, c: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);
            let b_value = ArgValue::from_obj(&b);
            let c_value = ArgValue::from_obj(&c);

            if !<$a as ArgTrait>::coercable(a_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #1, found <{}>",
                        <$a as ArgTrait>::ty(),
                        a_value.ty()
                    ),
                );
            }

            if !<$b as ArgTrait>::coercable(b_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #2, found <{}>",
                        <$b as ArgTrait>::ty(),
                        b_value.ty()
                    ),
                );
            }

            if !<$c as ArgTrait>::coercable(c_value.ty()) {
                raise_type_error(
                    token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #3, found <{}>",
                        <$c as ArgTrait>::ty(),
                        c_value.ty()
                    ),
                );
            }

            unsafe {
                $f(
                    <$a as ArgTrait>::from_arg_value(a_value).unwrap_unchecked(),
                    <$b as ArgTrait>::from_arg_value(b_value).unwrap_unchecked(),
                    <$c as ArgTrait>::from_arg_value(c_value).unwrap_unchecked(),
                )
            }
        }

        Fun3::new(trampoline)
    }};
}

macro_rules! fun_var_between {
    ($f:expr, $args_min:expr, $args_max:expr) => {{
        use ::micropython_rs::{fun::FunVarBetween, obj::Obj};

        unsafe extern "C" fn trampoline<'a>(n_args: usize, ptr: *const Obj) -> Obj {
            let f: fn(&'a [Obj]) -> Obj = $f;
            let args = unsafe { ::std::slice::from_raw_parts(ptr, n_args) };
            f(args)
        }

        FunVarBetween::new(trampoline, $args_min, $args_max)
    }};
}

macro_rules! fun_var_kw {
    ($f:expr, $args_min:expr) => {{
        use ::micropython_rs::fun::FunVarKw;

        unsafe extern "C" fn trampoline<'a>(n_args: usize, ptr: *const Obj, map: *mut Map) -> Obj {
            let f: fn(&'a [Obj], &'a Map) -> Obj = $f;
            let pos_args = unsafe { ::std::slice::from_raw_parts(ptr, n_args) };
            let kw_map = unsafe { &*map };
            f(pos_args, kw_map)
        }

        FunVarKw::new(trampoline, $args_min)
    }};
}

pub(crate) use fun_var_between;
pub(crate) use fun_var_kw;
pub(crate) use fun1;
pub(crate) use fun2;
pub(crate) use fun3;
