use std::time::Duration;

use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};

use crate::modvenice::vasyncio::time32::{MILLIS_PER_SEC, NANOS_PER_MILLI};

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

#[class(qstr!(TimeUnit))]
#[repr(C)]
pub struct TimeUnitObj {
    base: ObjBase<'static>,
    unit: TimeUnit,
}

#[class_methods]
impl TimeUnitObj {
    const fn new(unit: TimeUnit) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            unit,
        }
    }

    #[constant]
    pub const MILLIS: &Self = &Self::new(TimeUnit::Millis);
    #[constant]
    pub const SECOND: &Self = &Self::new(TimeUnit::Second);

    pub const fn unit(&self) -> &TimeUnit {
        &self.unit
    }
}
