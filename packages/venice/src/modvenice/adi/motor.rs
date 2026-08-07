use std::cell::RefCell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::motor::AdiMotor;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Motor Controller
///
/// This class provides an interface for controlling motors over ADI using PWM (Pulse-width
/// Modulation) output. ADI motor control is typically done using a physical hardware component
/// between the brain and the motor itself such as the [Motor Controller 29] to drive the motor.
///
/// # Hardware Overview
///
/// The two primary motors that this class is intended to control are the legacy cortex-era [Motor
/// 393] and Motor 269 units from VEX. These are fairly standard DC motors that can be driven using
/// standard voltage control or PWM, with an integrated PTC breaker designed to prevent damage to
/// the motors in the event that they are overcurrent or stalled.
///
/// While this class provides an API similar to that of a Smart Motor, it is in reality simply
/// outputting an 8-bit PWM signal, which will be processed by an intermediate motor controller
/// (such as the MC29) to drive the motor using an H-bridge circuit, allowing operation in either
/// direction.
///
/// Because these motors are no longer V5RC legal, they are not affected by competition control
/// restrictions, nor do they have any software-imposed current limitations beyond the
/// aforementioned PTC circuit.
///
/// [Motor Controller 29]: https://www.vexrobotics.com/276-2193.html
/// [Motor 393]: https://www.vexrobotics.com/393-motors.html
///
/// Cortex-era Motor Controller
#[class(qstr!(AdiMotor))]
#[repr(C)]
pub struct AdiMotorObj {
    base: ObjBase,
    motor: RefCell<AdiMotor>,
}

#[class_methods]
impl AdiMotorObj {
    /// Create a new motor from an ADI port.
    ///
    /// Motors can be optionally configured to use slew rate control to prevent the internal PTC
    /// from tripping on older cortex-era 393 motors.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    /// Pass `True` for `slew` to enable slew-rate limiting or `False` to disable it. The generated
    /// signature incorrectly declares `slew` as `float`, but the runtime accepts only booleans.
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Set the motor output to 50% power.
    /// motor.set_output(0.5)
    ///
    /// # Get the current motor output.
    /// output = motor.get_output()
    /// print("Current motor output:", output)
    ///
    /// # Stop the motor.
    /// motor.stop()
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    /// - `TypeError`: If `slew` isn't a boolean.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort, slew: float) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);
        let port = reader.next_positional_with(AdiPortParser)?;
        // TODO: should this be made optional? If so, what should be its default value?
        let slew = reader.next_positional()?;

        Ok(Self {
            base: ty.into(),
            motor: RefCell::new(AdiMotor::new(port, slew)),
        })
    }

    /// Sets the PWM output of the given motor to `value`, a floating point number in the range
    /// [-1.0, 1.0].
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Set the motor output to 50% power.
    /// motor.set_output(0.5)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_output(&self, value: f32) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().set_output(value as f64)?)
    }

    /// Sets the PWM output of the given motor as an integer `pwm` in the range [-127, 127].
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Set the motor output to 100 out of 127.
    /// motor.set_raw_output(100)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_raw_output(&self, pwm: i8) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().set_raw_output(pwm)?)
    }

    /// Returns the last set PWM output of the motor on the given port as a floating point number in
    /// the range [-1.0, 1.0].
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Get the current motor output.
    /// output = motor.get_output()
    /// print("Current motor output:", output)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_output(&self) -> Result<f32, Exception> {
        Ok(self.motor.borrow().output()? as f32)
    }

    /// Returns the last set PWM output of the motor on the given port as an integer in the range
    /// [-127, 127].
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Get the current motor output.
    /// output = motor.get_raw_output()
    /// print("Current motor output out of 127:", output)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_raw_output(&self) -> Result<i32, Exception> {
        Ok(self.motor.borrow().raw_output()? as i32)
    }

    /// Stops the given motor by setting its output to zero.
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new ADI motor on ADI port A with slew rate control enabled.
    /// motor = AdiMotor("A", True)
    ///
    /// # Stop the motor.
    /// motor.stop()
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn stop(&self) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().stop()?)
    }
}
