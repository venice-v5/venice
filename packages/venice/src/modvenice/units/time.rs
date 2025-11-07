use std::time::Duration;

use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use crate::qstrgen::qstr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Millis,
    Second,
}

impl TimeUnit {
    pub fn from_float(self, value: f32) -> Duration {
        let ms = match self {
            Self::Millis => value as u64,
            Self::Second => (value * 1000.0) as u64,
        };
        Duration::from_millis(ms)
    }
}

#[repr(C)]
pub struct TimeUnitObj {
    base: ObjBase,
    unit: TimeUnit,
}

static TIME_UNIT_OBJ_OBJ: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_slot_locals_dict_from_static(&const_dict![
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

    pub const fn new_static(unit: TimeUnit) -> &'static Self {
        match unit {
            TimeUnit::Millis => &Self::MILLIS,
            TimeUnit::Second => &Self::SECOND,
        }
    }

    pub const fn unit(&self) -> &TimeUnit {
        &self.unit
    }
}
