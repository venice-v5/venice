use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanoseconds(u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    secs_hi: u32,
    secs_lo: u32,
    nanos: Nanoseconds,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    inner: Duration,
}

const NANOS_PER_MICROS: u64 = 1000;
const MICROS_PER_SEC: u64 = 1_000_000;
const NANOS_PER_SEC: u64 = NANOS_PER_MICROS * MICROS_PER_SEC;

impl Nanoseconds {
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let ns = self.0 + rhs.0;
        if ns >= NANOS_PER_SEC as u32 {
            (Self(ns - NANOS_PER_SEC as u32), true)
        } else {
            (Self(ns), false)
        }
    }
}

impl Duration {
    pub const fn new(secs: u64, nanos: Nanoseconds) -> Self {
        Self {
            secs_hi: (secs << 32) as u32,
            secs_lo: secs as u32,
            nanos,
        }
    }

    pub const fn from_duration(duration: std::time::Duration) -> Self {
        Self::new(duration.as_secs(), Nanoseconds(duration.subsec_nanos()))
    }

    pub const fn secs(&self) -> u64 {
        (self.secs_lo as u64) | (self.secs_hi as u64) << 32
    }

    pub const fn from_micros(micros: u64) -> Self {
        let mut nanos = micros * NANOS_PER_MICROS;
        let mut secs = micros / MICROS_PER_SEC;

        if nanos >= NANOS_PER_SEC {
            let overflow = nanos / NANOS_PER_SEC;
            nanos = nanos % NANOS_PER_SEC;
            secs += overflow;
        }

        Self::new(secs, Nanoseconds(nanos as u32))
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let (nanos, carry) = self.nanos.overflowing_add(rhs.nanos);
        let secs = self.secs() + rhs.secs() + if carry { 1 } else { 0 };
        Self::new(secs, nanos)
    }
}

impl Instant {
    const fn from_duration(dur: Duration) -> Self {
        Self { inner: dur }
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
        Self {
            inner: self.inner + rhs,
        }
    }
}
