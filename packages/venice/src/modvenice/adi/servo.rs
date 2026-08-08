use std::cell::RefCell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::servo::AdiServo;

use crate::modvenice::{Exception, adi::expander::AdiPortParser, units::rotation::RotationUnitObj};

/// ADI Servo
///
/// This class provides an interface for controlling the legacy 3-Wire Servo.
///
/// # Hardware Overview
///
/// Servos are similar in both appearance and function to `AdiMotor`s,
/// with the caveat that they are designed to hold a specific *angle* rather than a continuous
/// *speed*. In other words:
///
/// - Motors are designed for continuous rotation, providing variable speed and direction of
///   rotation.
/// - Servos are designed for precise angular positioning, typically rotating to and holding a
///   specific angle within a limited range of motion.
///
/// Servos, similar to motors, are PWM controlled. They use a standard
/// [servo control](https://en.wikipedia.org/wiki/Servo_control) signal. A PWM input of
/// 1ms - 2ms will give full reverse to full forward, while 1.5ms is neutral.
///
/// # Operating Range
///
/// The VEX legacy servo has an operating range of 100 degrees:
/// - Minimum: -50 degrees (represented by `AdiServo.MIN_POSITION_DEG`)
/// - Maximum: 50 degrees (represented by `AdiServo.MAX_POSITION_DEG`)
///
/// Its neutral state is at 0° rotation (the middle of its operating range).
///
/// Legacy Servo
#[class(qstr!(AdiServo))]
pub struct AdiServoObj {
    base: ObjBase,
    servo: RefCell<AdiServo>,
}

#[class_methods]
impl AdiServoObj {
    /// Minimum controllable position of the servo in degrees.
    #[constant]
    const MIN_POSITION_DEG: f32 = AdiServo::MIN_POSITION.as_degrees() as f32;
    /// Maximum controllable position of the servo in degrees.
    #[constant]
    const MAX_POSITION_DEG: f32 = AdiServo::MAX_POSITION.as_degrees() as f32;

    /// Creates a servo from an ADI port.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// servo = AdiServo("A")
    /// servo.set_target(25, DEGREES)
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
            servo: RefCell::new(AdiServo::new(port)),
        })
    }

    /// Sets the servo's position target in the supplied `unit`.
    ///
    /// # Range
    ///
    /// VEX servos have an operating range of 100° spanning from `AdiServo.MIN_POSITION_DEG` (-50°) to
    /// `AdiServo.MAX_POSITION_DEG` (50°). Values outside of this range will be saturated at their
    /// respective min or max value.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// servo = AdiServo("A")
    /// servo.set_target(25, DEGREES)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_target(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .servo
            .borrow_mut()
            .set_target(unit.unit().float_to_angle(position))?)
    }

    /// Sets the servo's raw position using a raw 8-bit PWM input `pwm` from [-127, 127]. This is
    /// functionally equivalent to `AdiServo.set_target` with the exception that it accepts an
    /// unscaled integer rather than a value in a `RotationUnit`.
    ///
    /// # Range
    ///
    /// The raw input spans from -127 to 127. A value of -127 corresponds to
    /// `AdiServo.MIN_POSITION_DEG` (-50°), zero is centered, and 127 corresponds to
    /// `AdiServo.MAX_POSITION_DEG` (50°).
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// servo = AdiServo("A")
    /// # Set the servo to the center position.
    /// servo.set_raw_target(0)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_raw_target(&self, pwm: i8) -> Result<(), Exception> {
        Ok(self.servo.borrow_mut().set_raw_target(pwm)?)
    }
}
