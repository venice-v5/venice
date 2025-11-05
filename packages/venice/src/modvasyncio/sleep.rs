use std::time::{Duration, Instant};

use cty::c_void;
use micropython_rs::{
    except::{raise_stop_iteration, raise_type_error},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use crate::{obj::alloc_obj, qstrgen::qstr};

#[repr(C)]
pub struct Sleep {
    base: ObjBase,
    deadline: Instant,
}

pub static SLEEP_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(Sleep))
        .set_slot_iter(sleep_iternext as *const c_void)
        .set_slot_make_new(sleep_make_new);

unsafe impl ObjTrait for Sleep {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = SLEEP_OBJ_TYPE.as_obj_type();
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            deadline: Instant::now() + duration,
        }
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}

extern "C" fn sleep_make_new(
    _: *const ObjType,
    n_args: usize,
    n_kw: usize,
    args: *const Obj,
) -> Obj {
    let token = token().unwrap();
    if n_args != 0 {
        raise_type_error(
            token,
            format!("function doesn't take positional arguments, but {n_args} were provided"),
        );
    }

    if n_kw < 1 {
        raise_type_error(
            token,
            "expected at least one keyword argument (either `millis=n` or `secs=n`)",
        );
    }

    let slice = unsafe { std::slice::from_raw_parts(args, n_kw * 2) };
    let arg_name = slice[0].get_str().unwrap();
    let value = slice[1]
        .try_to_int()
        .unwrap_or_else(|| raise_type_error(token, "expected integer"));

    let duration = match arg_name {
        b"millis" => Duration::from_millis(value as u64),
        b"secs" => Duration::from_secs(value as u64),
        _ => raise_type_error(
            token,
            "invalid keyword argument (expected either `millis=n` or `secs=n`)",
        ),
    };

    alloc_obj(Sleep::new(duration))
}

extern "C" fn sleep_iternext(self_in: Obj) -> Obj {
    let sleep = self_in.try_to_obj::<Sleep>().unwrap();
    if sleep.deadline <= Instant::now() {
        raise_stop_iteration(token().unwrap(), Obj::NONE);
    } else {
        self_in
    }
}
