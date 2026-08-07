use std::cell::RefCell;

use argparse::{Args, PositionalError};
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::digital::{AdiDigitalIn, AdiDigitalOut, LogicLevel};

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Digital Input
///
/// ADI ports on the V5 brain are capable of sending and receiving digital signals with external
/// devices. Digital signals represent binary information using voltage levels (called logic levels) - they can only be in one of two states at any time. Unlike analog
/// signals which can take on any voltage within a range, digital signals are either fully "on"
/// (high) or fully "off" (low), making them ideal for simple sensors and actuators such as buttons,
/// switches and solenoids.
///
/// # Hardware Description
///
/// `AdiDigitalIn` configures an ADI (Analog/Digital Interface) port as a digital input. It detects
/// voltage levels to determine a logical high (3.3V or above) or low (below 3.3V) state. Digital
/// inputs can use either direct Brain connections or an ADI expander.
///
/// Generic digital input over ADI.
///
/// Represents an ADI port configured to receive digital input. The pin can be read to determine its
/// current logic level (`True` for high or `False` for low (above or below 3.3V)).
#[class(qstr!(AdiDigitalIn))]
#[repr(C)]
pub struct AdiDigitalInObj {
    base: ObjBase,
    r#in: AdiDigitalIn,
}

/// Generic digital output over ADI.
///
/// Represents an ADI port configured to send digital signals to a device. The output drives the pin
/// to either 3.3V (high) or 0V (low). This can be used for toggling solenoids or other external
/// devices that might need a digital signal from the Brain.
#[class(qstr!(AdiDigitalOut))]
#[repr(C)]
pub struct AdiDigitalOutObj {
    base: ObjBase,
    out: RefCell<AdiDigitalOut>,
}

fn level_to_bool(level: LogicLevel) -> bool {
    match level {
        LogicLevel::High => true,
        LogicLevel::Low => false,
    }
}

fn bool_to_level(b: bool) -> LogicLevel {
    match b {
        true => LogicLevel::High,
        false => LogicLevel::Low,
    }
}

#[class_methods]
impl AdiDigitalInObj {
    /// Creates a digital input from an ADI port.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        Ok(Self {
            base: ty.into(),
            r#in: AdiDigitalIn::new(port),
        })
    }

    /// Returns the current logic level of a digital input pin as `True` for high or `False` for low.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_value(&self) -> Result<bool, Exception> {
        Ok(level_to_bool(self.r#in.level()?))
    }

    /// Returns `True` if the digital input's logic level is high.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn is_high(&self) -> Result<bool, Exception> {
        Ok(self.r#in.is_high()?)
    }

    /// Returns `True` if the digital input's logic level is low.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn is_low(&self) -> Result<bool, Exception> {
        Ok(self.r#in.is_low()?)
    }
}

#[class_methods]
impl AdiDigitalOutObj {
    /// Creates a digital output from an ADI port, optionally with an initial logic level.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    /// `initial_level` defaults to `None`, which leaves the output at the device's default level. The
    /// current binding accepts exactly one positional argument and no keywords, so explicitly supplying
    /// `True` or `False` is not reachable.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     digital_out = AdiDigitalOut("A")
    ///
    ///     # Toggle the digital output every second.
    ///     while True:
    ///         digital_out.toggle()
    ///         await vasyncio.Sleep(1, SECOND)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `initial_level` is explicitly supplied.
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort, initial_level: bool | None = None) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        let initial_level = match reader.next_positional::<bool>() {
            Ok(v) => Some(v),
            Err(e) => match e {
                PositionalError::ArgumentsExhausted => None,
                _ => Err(e)?,
            },
        };

        let out = match initial_level {
            Some(level) => AdiDigitalOut::with_initial_level(port, bool_to_level(level)),
            None => AdiDigitalOut::new(port),
        };

        Ok(Self {
            base: ty.into(),
            out: RefCell::new(out),
        })
    }

    /// Returns the current set logic level of a digital output pin as `True` for high or `False` for low.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_value(&self) -> Result<bool, Exception> {
        Ok(level_to_bool(self.out.borrow().level()?))
    }

    /// Sets the digital logic level (high or low) of a pin. `value=True` selects high and
    /// `value=False` selects low.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_value(&self, value: bool) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_level(bool_to_level(value))?)
    }

    /// Sets the digital logic level to high. This is analogous to `set_value(True)`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_high(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_high()?)
    }

    /// Sets the digital logic level to low. This is analogous to `set_value(False)`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_low(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_low()?)
    }

    /// Sets the digital logic level to the inverse of its previous state.
    ///
    /// - If the port was previously set to low, then the level will be set to high.
    /// - If the port was previously set to high, then the level will be set to low.
    ///
    /// This is analogous to `set_value(not get_value())` and is useful for toggling devices like
    /// solenoids.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     digital_out = AdiDigitalOut("A")
    ///
    ///     # Toggle the digital output every second.
    ///     while True:
    ///         digital_out.toggle()
    ///         await vasyncio.Sleep(1, SECOND)
    ///
    /// vasyncio.run(main())
    /// ```
    #[method]
    fn toggle(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().toggle()?)
    }
}
