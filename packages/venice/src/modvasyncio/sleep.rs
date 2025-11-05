use std::time::Duration;

use cty::c_void;
use micropython_rs::{
    except::{raise_stop_iteration, raise_type_error},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use crate::{
    args::{ArgType, Args},
    obj::alloc_obj,
    qstrgen::qstr,
};

#[repr(C)]
pub struct Sleep {
    base: ObjBase,
    deadline: super::instant::Instant,
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
            deadline: super::instant::Instant::now() + duration,
        }
    }

    pub fn deadline(&self) -> super::instant::Instant {
        self.deadline
    }
}

extern "C" fn sleep_make_new(_: *const ObjType, n_pos: usize, n_kw: usize, ptr: *const Obj) -> Obj {
    let token = token().unwrap();
    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, ptr) }.reader(token);
    args.assert_npos(0, 0).assert_nkw(1, 1);

    let arg = args.next_kw(ArgType::Int);
    let val = arg.value.as_int() as u64;
    let duration = match arg.kw {
        b"millis" => Duration::from_millis(val),
        b"secs" => Duration::from_secs(val),
        _ => raise_type_error(
            token,
            "invalid keyword argument (expected either `millis=n` or `secs=n`)",
        ),
    };

    alloc_obj(Sleep::new(duration))
}

extern "C" fn sleep_iternext(self_in: Obj) -> Obj {
    let sleep = self_in.try_to_obj::<Sleep>().unwrap();
    if sleep.deadline <= super::instant::Instant::now() {
        raise_stop_iteration(token().unwrap(), Obj::NONE);
    } else {
        self_in
    }
}
