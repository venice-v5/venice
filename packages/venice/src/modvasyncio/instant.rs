use std::{ops::Add, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    secs_hi: u32,
    secs_lo: u32,
    nanos: u32,
}

impl Instant {
    const fn to_duration(&self) -> Duration {
        Duration::new(
            ((self.secs_hi as u64) << 32) | (self.secs_lo as u64),
            self.nanos,
        )
    }

    const fn from_duration(dur: Duration) -> Self {
        Self {
            secs_hi: (dur.as_secs() >> 32) as u32,
            secs_lo: dur.as_secs() as u32,
            nanos: dur.subsec_nanos(),
        }
    }

    pub fn now() -> Self {
        Self::from_duration(Duration::from_micros(unsafe {
            vex_sdk::vexSystemHighResTimeGet()
        }))
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_duration(self.to_duration() + rhs)
    }
}
