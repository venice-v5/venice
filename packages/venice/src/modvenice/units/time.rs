use std::time::Duration;

use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use crate::{
    modvenice::vasyncio::time32::{MILLIS_PER_SEC, NANOS_PER_MILLI},
    qstrgen::qstr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Millis,
    Second,
}

impl TimeUnit {
    pub fn float_to_dur(self, value: f32) -> Duration {
        let secs = match self {
            Self::Millis => value / 1000.0,
            Self::Second => value,
        };
        Duration::from_secs_f32(secs)
    }

    pub fn dur_to_float(self, dur: Duration) -> f32 {
        match self {
            Self::Second => dur.as_secs_f32(),
            Self::Millis => {
                (dur.as_secs() as f32) * (MILLIS_PER_SEC as f32)
                    + (dur.subsec_nanos() as f32) / (NANOS_PER_MILLI as f32)
            }
        }
    }
}

#[repr(C)]
pub struct TimeUnitObj {
    base: ObjBase<'static>,
    unit: TimeUnit,
}

static TIME_UNIT_OBJ_OBJ: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_locals_dict(const_dict![
        qstr!(MILLIS) => Obj::from_static(&TimeUnitObj::MILLIS),
        qstr!(SECOND) => Obj::from_static(&TimeUnitObj::SECOND),
    ]);

unsafe impl ObjTrait for TimeUnitObj {
    const OBJ_TYPE: &ObjType = TIME_UNIT_OBJ_OBJ.as_obj_type();
}

impl TimeUnitObj {
    pub const MILLIS: Self = Self::new(TimeUnit::Millis);
    pub const SECOND: Self = Self::new(TimeUnit::Second);

    pub const fn new(unit: TimeUnit) -> Self {
        Self {
            base: ObjBase::new(TIME_UNIT_OBJ_OBJ.as_obj_type()),
            unit,
        }
    }

    pub const fn unit(&self) -> &TimeUnit {
        &self.unit
    }
}
