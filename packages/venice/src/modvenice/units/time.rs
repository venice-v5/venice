use std::time::Duration;

use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::value_error,
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};

use crate::modvenice::{
    Exception,
    vasyncio::time32::{MILLIS_PER_SEC, NANOS_PER_MILLI},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Millis,
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeConversionError {
    Negative,
    NonFinite,
    OutOfRange,
}

impl From<TimeConversionError> for Exception {
    fn from(error: TimeConversionError) -> Self {
        value_error(match error {
            TimeConversionError::Negative => c"time value cannot be negative",
            TimeConversionError::NonFinite => c"time value must be finite",
            TimeConversionError::OutOfRange => c"time value is too large",
        })
        .into()
    }
}

impl TimeUnit {
    pub fn float_to_dur(self, value: f32) -> Result<Duration, TimeConversionError> {
        if value.is_nan() || value.is_infinite() {
            return Err(TimeConversionError::NonFinite);
        }
        if value < 0.0 {
            return Err(TimeConversionError::Negative);
        }

        let secs = match self {
            Self::Millis => value / 1000.0,
            Self::Second => value,
        };
        Duration::try_from_secs_f32(secs).map_err(|_| TimeConversionError::OutOfRange)
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

/// A unit selector for time intervals and durations.
///
/// Venice time APIs interpret or return numeric values in the selected unit. One second equals 1,000
/// milliseconds.
///
/// This class is not constructed directly. Use `TimeUnit.MILLIS` or `TimeUnit.SECOND`, which are also
/// exported at the package root as `MILLIS` and the singular `SECOND`. Values have readable
/// representations such as `TimeUnit.SECOND`.
#[class(qstr!(TimeUnit))]
#[repr(C)]
pub struct TimeUnitObj {
    base: ObjBase,
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

    /// Selects milliseconds; also root-importable as `MILLIS`.
    #[constant]
    pub const MILLIS: &Self = &Self::new(TimeUnit::Millis);
    /// Selects seconds; also root-importable as the singular name `SECOND`.
    #[constant]
    pub const SECOND: &Self = &Self::new(TimeUnit::Second);

    pub const fn unit(&self) -> &TimeUnit {
        &self.unit
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.unit {
            TimeUnit::Millis => "TimeUnit.MILLIS",
            TimeUnit::Second => "TimeUnit.SECOND",
        })
    }
}
