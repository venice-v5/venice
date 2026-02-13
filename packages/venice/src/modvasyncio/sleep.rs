use std::cell::Cell;

use micropython_rs::{
    except::{raise_stop_iteration, raise_type_error},
    init::token,
    make_new_from_fn,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use super::time32;
use crate::{
    args::{ArgValue, Args},
    modvenice::units::time::TimeUnitObj,
    obj::alloc_obj,
    qstrgen::qstr,
};

#[repr(C)]
pub struct Sleep {
    base: ObjBase<'static>,
    duration: time32::Duration,
    complete: Cell<bool>,
}

pub static SLEEP_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::ITER_IS_ITERNEXT, qstr!(Sleep))
        .set_make_new(make_new_from_fn!(sleep_make_new))
        .set_iter(Iter::IterNext(sleep_iternext));

unsafe impl ObjTrait for Sleep {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = SLEEP_OBJ_TYPE.as_obj_type();
}

impl Sleep {
    pub fn new(duration: time32::Duration) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            duration,
            complete: Cell::new(false),
        }
    }

    pub fn duration(&self) -> time32::Duration {
        self.duration
    }

    pub fn complete(&self) {
        self.complete.set(true);
    }
}

fn sleep_make_new(_: &ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token();
    let mut args = Args::new(n_pos, n_kw, args).reader(token);
    args.assert_npos(2, 2);

    let interval_float = match args.try_next_positional_untyped() {
        Ok(arg) => match arg {
            ArgValue::Float(float) => float,
            ArgValue::Int(int) => int as f32,
            _ => raise_type_error(
                token,
                format!(
                    "expected <float> or <int> for argument #1, found <{}>",
                    arg.ty()
                ),
            ),
        },
        // only error possible is PositionalsExhuasted, but that isn't reachable because of the arg
        // count assertion
        Err(_) => unreachable!(),
    };

    let unit = args.next_positional::<&TimeUnitObj>().unit();
    let duration = time32::Duration::from_duration(unit.float_to_dur(interval_float));
    alloc_obj(Sleep::new(duration))
}

extern "C" fn sleep_iternext(self_in: Obj) -> Obj {
    let sleep = self_in.try_as_obj::<Sleep>().unwrap();
    if sleep.complete.get() {
        raise_stop_iteration(token(), Obj::NONE);
    } else {
        self_in
    }
}
