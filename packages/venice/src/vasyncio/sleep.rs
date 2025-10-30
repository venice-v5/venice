use std::time::{Duration, Instant};

use cty::c_void;
use micropython_rs::{
    except::{raise_stop_iteration, raise_type_error},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};

use crate::{obj::alloc_obj, qstrgen::qstr};

#[repr(C)]
pub struct Sleep {
    base: ObjBase,
    deadline: Instant,
}

static SLEEP_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(Sleep))
    .set_slot_iter(sleep_iternext as *const c_void);

unsafe impl ObjTrait for Sleep {
    const OBJ_TYPE: *const micropython_rs::obj::ObjType = SLEEP_OBJ_TYPE.as_obj_type_ptr();
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        Self {
            base: ObjBase::new::<Self>(),
            deadline: Instant::now() + duration,
        }
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}

pub extern "C" fn sleep_ms(ms: Obj) -> Obj {
    let ms = match ms.as_small_int() {
        Some(ms) => ms,
        None => raise_type_error(
            token().unwrap(),
            "expected integer, got (TODO?: print type received)",
        ),
    };
    alloc_obj(Sleep::new(Duration::from_millis(ms as u64)))
}

extern "C" fn sleep_iternext(self_in: Obj) -> Obj {
    let sleep = self_in.as_obj::<Sleep>().unwrap();
    if sleep.deadline <= Instant::now() {
        raise_stop_iteration(token().unwrap(), Obj::NONE);
    } else {
        self_in
    }
}
