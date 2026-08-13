use std::cell::RefCell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::pwm::AdiPwmOut;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Pulse-width Modulation (PWM)
///
/// This class provides an interface for generating 8-bit PWM signals through ADI ports.
///
/// # Hardware Overview
///
/// Pulse-width modulation (PWM) is a digital signaling technique that creates a variable width high
/// pulse over a fixed period, allowing you to communicate analog data over digital signals by
/// measuring the length of the pulse (how long it was high compared to how long it was low).
///
/// PWM signals consist of two components:
/// - ON time (pulse width): When the signal is high (3.3V)
/// - OFF time: When the signal is low (0V)
///
/// The ratio between ON time and OFF time (the "duty cycle") is used to encode
/// information for commands to devices:
///
/// ```text
///             |<-->| pulse width (0.94-2.03mS)
/// 3.3V  ┐     ┌────┐     ┌──┐       ┌──────┐
/// 0V    └─────┘    └─────┘  └───────┘      └────
///             |<-------->| period (16mS)
/// ```
///
/// Generic PWM Output over ADI
#[class(qstr!(AdiPwmOut))]
pub struct AdiPwmOutObj {
    base: ObjBase,
    pwm: RefCell<AdiPwmOut>,
}

#[class_methods]
impl AdiPwmOutObj {
    /// Creates a PWM output from an ADI port.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// pwm = AdiPwmOut("A")
    /// pwm.set_output(128)  # Set PWM to 50% duty cycle.
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort, /) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?.commit()?;
        Ok(Self {
            base: ty.into(),
            pwm: RefCell::new(AdiPwmOut::new(port)),
        })
    }

    /// Sets the PWM output width.
    ///
    /// `value` must be an integer from `0` through `255`. This value is sent over 16ms periods with
    /// pulse widths ranging from roughly 0.94mS to 2.03mS.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// pwm = AdiPwmOut("A")
    /// pwm.set_output(128)  # Set PWM to 50% duty cycle.
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `value` is outside the range from `0` through `255`.
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_output(&self, value: u8) -> Result<(), Exception> {
        Ok(self.pwm.borrow_mut().set_output(value)?)
    }
}
