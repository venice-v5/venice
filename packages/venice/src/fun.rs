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

pub(crate) use fun_var_kw;
