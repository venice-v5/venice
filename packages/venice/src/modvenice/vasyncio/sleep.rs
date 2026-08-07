use std::cell::Cell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::raise_stop_iteration,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};

use super::time32;
use crate::modvenice::{Exception, units::time::TimeUnitObj};

/// An awaitable that will complete after a given duration.
///
/// Awaiting a `Sleep` effectively yields the current task for a period of time. Constructing it does
/// not block and does not start a separate task. When the duration has elapsed, awaiting it returns
/// `None`.
#[class(qstr!(Sleep))]
#[repr(C)]
pub struct Sleep {
    base: ObjBase,
    duration: time32::Duration,
    complete: Cell<bool>,
}

impl Sleep {
    pub fn new(duration: time32::Duration) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
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

#[class_methods]
impl Sleep {
    /// Waits until `interval`, measured in `unit`, has elapsed.
    ///
    /// This constructor returns an awaitable that will complete after the given duration, effectively
    /// yielding the current task for a period of time. Use `MILLIS` for milliseconds or `SECOND` for
    /// seconds. `interval` should be finite and non-negative; the binding currently relies on the
    /// underlying duration conversion rather than raising a Python exception for invalid values.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     print("See you in 5 minutes.")
    ///     await vasyncio.Sleep(300, SECOND)
    ///     print("Hello again!")
    ///
    /// vasyncio.run(main())
    /// ```
    #[make_new]
    #[stub(sig = "(self, interval: float, unit: TimeUnit) -> None")]
    fn make_new(_: &ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Result<Self, Exception> {
        let mut args = Args::new(n_pos, n_kw, args).reader();
        args.assert_npos(2, 2);

        let interval = args.next_positional()?;
        let unit = args.next_positional::<&TimeUnitObj>()?.unit();

        let duration = time32::Duration::from_duration(unit.float_to_dur(interval));
        Ok(Self::new(duration))
    }

    #[iter]
    extern "C" fn sleep_iternext(self_in: Obj) -> Obj {
        let sleep = self_in.as_obj::<Sleep>();
        if sleep.complete.get() {
            raise_stop_iteration(token(), Obj::NONE);
        } else {
            self_in
        }
    }
}
