macro_rules! fun1_from_fn {
    ($f:expr, $a:ty) => {{
        use ::micropython_rs::{except::raise_type_error, fun::Fun1, init::token, obj::Obj};
        use $crate::args::{ArgTrait, ArgValue};

        extern "C" fn trampoline(a: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);

            if a_value.ty() != <$a as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
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

macro_rules! fun2_from_fn {
    ($f:expr, $a:ty, $b:ty) => {{
        use ::micropython_rs::{except::raise_type_error, fun::Fun2, init::token, obj::Obj};
        use $crate::args::{ArgTrait, ArgValue};

        extern "C" fn trampoline(a: Obj, b: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);
            let b_value = ArgValue::from_obj(&b);

            if a_value.ty() != <$a as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
                        "expected <{}> for argument #1, found <{}>",
                        <$a as ArgTrait>::ty(),
                        a_value.ty()
                    ),
                );
            }

            if b_value.ty() != <$b as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
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

macro_rules! fun3_from_fn {
    ($f:expr, $a:ty, $b:ty, $c:ty) => {{
        use ::micropython_rs::{except::raise_type_error, fun::Fun3, init::token, obj::Obj};
        use $crate::args::{ArgTrait, ArgValue};

        extern "C" fn trampoline(a: Obj, b: Obj, c: Obj) -> Obj {
            let a_value = ArgValue::from_obj(&a);
            let b_value = ArgValue::from_obj(&b);
            let c_value = ArgValue::from_obj(&c);

            if a_value.ty() != <$a as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
                        "expected <{}> for argument #1, found <{}>",
                        <$a as ArgTrait>::ty(),
                        a_value.ty()
                    ),
                );
            }

            if b_value.ty() != <$b as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
                        "expected <{}> for argument #2, found <{}>",
                        <$b as ArgTrait>::ty(),
                        b_value.ty()
                    ),
                );
            }

            if c_value.ty() != <$c as ArgTrait>::ty() {
                raise_type_error(
                    token().unwrap(),
                    format!(
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

macro_rules! fun_var_from_fn {
    ($f:expr) => {{
        use ::micropython_rs::fun::FunVar;

        unsafe extern "C" fn trampoline(n_args: usize, ptr: *const Obj) -> Obj {
            let args = unsafe { ::std::slice::from_raw_parts(ptr, n_args) };
            $f(args)
        }

        FunVar::new(trampoline)
    }};
}

pub(crate) use fun_var_from_fn;
pub(crate) use fun1_from_fn;
pub(crate) use fun2_from_fn;
pub(crate) use fun3_from_fn;
