use std::cell::RefCell;

use argparse::{Args, IntParser, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::value_error,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::encoder::AdiEncoder;

use crate::modvenice::{
    Exception,
    adi::{
        adi_port_name,
        expander::{AdiPortParser, AdiPortSpec, commit_adi_port_pair},
        expander_index,
    },
    units::rotation::RotationUnitObj,
};

/// ADI Shaft Encoder
///
/// This class provides an interface to interact with three-wire encoders, which are used to measure
/// both the relative position of and rotational distance traveled by a shaft.
///
/// In addition to the [VEX Optical Shaft Encoder](https://www.vexrobotics.com/276-2156.html), this
/// class also supports custom three-wire encoders with custom resolutions (TPR).
///
/// # Hardware Overview (for the Optical Shaft Encoder)
///
/// The Optical Shaft Encoder can be used to track distance traveled, direction of motion, or
/// position of any rotary component, such as a gripper arm or tracking wheel.
///
/// The encoder works by shining light onto the edge of a disk outfitted with evenly spaced slits
/// around the circumference. As the disk spins, light passes through the slits and is blocked by
/// the opaque spaces between the slits. The encoder then detects how many slits have
/// had light shine through, and in which direction the disk is spinning.
///
/// The encoder can detect up to 1,700 pulses per second, which corresponds to 18.9 revolutions per
/// second and 1,133 rpm (revolutions per minute). Faster revolutions will therefore not be
/// interpreted exactly, potentially resulting in erroneous positional data being returned.
///
/// ## Connecting to the V5 Brain
///
/// Encoders are two-wire devices that must be connected to two adjacent ports on the same brain/ADI
/// expander. One of the wires must be plugged into an odd-numbered port (A, C, E, G), while the
/// other wire must be plugged into the port directly above that wire (that is, B, D, F, or H,
/// respectively). If the top wire is plugged into the lower odd-numbered port (A, C, E, G), then
/// *clockwise* rotation will represent a positive change in position. If the bottom wire is plugged
/// into the lower port, then *counterclockwise* rotation will be positive instead.
///
/// # Comparison to `RotationSensor`
///
/// Rotation sensors and Shaft Encoders both measure the same thing (angular position), but with
/// some important differences. The largest distinction is how position is measured. Rotation
/// sensors use hall-effect magnets and know their absolute angle at any given time, including after
/// a power cycle or loss of voltage. In contrast, encoders only track their *change* in position,
/// meaning that any changes made to the encoder while unplugged will not be detected as a change in
/// position. Rotation sensors have much higher resolution than the old encoders sold by VEX at
/// 0.088° accuracy (compared to 1° of accuracy) and can measure accurately at higher
/// speeds. Rotation sensors are also capable of slotting VEX's new high-strength shafts, while
/// these older encoders can only fit low-strength shafts.
///
/// |                     | `AdiEncoder`       | `RotationSensor`                   |
/// | ------------------- | ------------------ | ---------------------------------- |
/// | Port                | Two ADI connections | One integer Smart Port             |
/// | Resolution          | 360 Ticks/Rev      | 4090 Ticks/Rev                     |
/// | Measurements        | Position           | Position, Absolute Angle, Velocity |
/// | Update Rate         | 10mS               | 10mS                               |
/// | Shaft Compatibility | Low Strength       | Low Strength, High Strength        |
///
/// ADI Shaft Encoder
#[class(qstr!(AdiEncoder))]
pub struct AdiEncoderObj {
    base: ObjBase,
    // vexide doesn't support non-const tpr values, so we have to set a tpr of 1 and manually make
    // tpr corrections
    // theta = (ticks * TAU) / tpr
    // ticks = (theta * tpr) / TAU
    encoder: RefCell<AdiEncoder<1>>,
    tpr: i32,
}

fn check_ports(top_port: AdiPortSpec<'_>, bottom_port: AdiPortSpec<'_>) -> Result<(), Exception> {
    if expander_index(top_port.expander_number()) != expander_index(bottom_port.expander_number()) {
        Err(value_error(error_msg!(
            "The specified top and bottom ports belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
            top_port.expander_number(),
            bottom_port.expander_number(),
        )))?;
    }

    let top_number = top_port.number();
    let bottom_number = bottom_port.number();
    let valid_combo = if top_number.is_multiple_of(2) {
        bottom_number == top_number - 1
    } else {
        bottom_number == top_number + 1
    };

    if !valid_combo {
        Err(value_error(error_msg!(
            "Encoder ports must be placed directly next to each other and in some combination of AB, CD, EF, GH, or BA, CD, EF, HG. (Got `{}{}`)",
            adi_port_name(top_number),
            adi_port_name(bottom_number),
        )))?;
    }

    Ok(())
}

#[class_methods]
impl AdiEncoderObj {
    /// Creates a new encoder with a given `tpr` from `top_port` and `bottom_port`.
    ///
    /// The ports must be an adjacent pair on the same Brain or `AdiExpander`: A/B, C/D, E/F, or G/H, in
    /// either order. Reversing their order reverses the positive direction. `tpr` is the encoder's ticks
    /// per revolution and defaults to 360 for the VEX Optical Shaft Encoder. It must be a positive
    /// integer.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     # Change to 360 if you're using the encoders sold by VEX.
    ///     encoder = AdiEncoder("A", "B", 8192)
    ///
    ///     while True:
    ///         print("encoder position:", encoder.get_position(DEGREES))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `tpr` is not positive, either port is invalid or occupied, the ports
    ///   belong to different ADI expanders, or they do not form an adjacent pair.
    #[make_new]
    #[stub(
        sig = "(self, top_port: str | AdiExpanderPort, bottom_port: str | AdiExpanderPort, tpr: int = 360, /) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 3).assert_nkw(0, 0);

        let top_port = reader.next_positional_with(AdiPortParser)?;
        let bottom_port = reader.next_positional_with(AdiPortParser)?;
        let tpr = match reader.next_positional_with(IntParser::new(1..=i32::MAX)) {
            Ok(tpr) => tpr,
            Err(argparse::PositionalError::ArgumentsExhausted) => 360,
            Err(error) => return Err(error.into()),
        };
        check_ports(top_port, bottom_port)?;
        let (top_port, bottom_port) = commit_adi_port_pair(top_port, bottom_port)?;

        Ok(Self {
            base: ty.into(),
            encoder: RefCell::new(AdiEncoder::new(top_port, bottom_port)),
            tpr,
        })
    }

    /// Returns the position reading of the encoder sensor in the supplied `unit`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     # Change to 360 if you're using the encoders sold by VEX.
    ///     encoder = AdiEncoder("A", "B", 8192)
    ///
    ///     while True:
    ///         print("encoder position:", encoder.get_position(DEGREES))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let tick_turns = self.encoder.borrow().position()?; // ticks * TAU
        let position = tick_turns / self.tpr as f64; // (ticks * TAU) / tpr
        Ok(unit.unit().angle_to_float(position))
    }

    /// Sets the current encoder position to the given position without any actual movement.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given
    /// position. This can be useful if you want to reset the encoder position to a known value
    /// at a certain point.
    ///
    /// `position` is interpreted in the supplied `unit`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Change to 360 if you're using the encoders sold by VEX.
    /// encoder = AdiEncoder("A", "B", 8192)
    ///
    /// # Treat the encoder as if it were at 180 degrees.
    /// encoder.set_position(180, DEGREES)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position); // theta
        let ticks = angle * self.tpr as f64; // theta * TPR
        Ok(self.encoder.borrow_mut().set_position(ticks)?) // function internally divides by TAU
    }

    /// Sets the current encoder position to zero.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given
    /// position.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Change to 360 if you're using the encoders sold by VEX.
    /// encoder = AdiEncoder("A", "B", 8192)
    ///
    /// # Reset the encoder position to zero.
    /// # This doesn't really do anything in this case, but it's a good example.
    /// encoder.reset_position()
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        Ok(self.encoder.borrow_mut().reset_position()?)
    }
}
