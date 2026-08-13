use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::adi::accelerometer::{AdiAccelerometer, Sensitivity};

use crate::modvenice::{Exception, adi::expander::AdiPortParser, read_only_attr::read_only_attr};

// TODO: remove the Adi prefix, PotentiometerType doesn't have it
/// The jumper setting of the accelerometer.
#[class(qstr!(AdiAccelerometerSensitivity))]
#[repr(C)]
pub struct SensitivityObj {
    base: ObjBase,
    sensitivity: Sensitivity,
}

#[class_methods]
impl SensitivityObj {
    const fn new(sensitivity: Sensitivity) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            sensitivity,
        }
    }

    /// 0-2g sensitivity.
    #[constant]
    pub const LOW: &Self = &Self::new(Sensitivity::Low);
    /// 0-6g sensitivity.
    #[constant]
    pub const HIGH: &Self = &Self::new(Sensitivity::High);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.sensitivity {
            Sensitivity::High => "AdiAccelerometerSensitivity.HIGH",
            Sensitivity::Low => "AdiAccelerometerSensitivity.LOW",
        });
    }
}

/// ADI Accelerometer
///
/// This class provides an interface for the LIS344ALH-based three-axis analog accelerometer.
///
/// # Hardware Overview
///
/// The LIS344ALH capacitive accelerometer features signal conditioning, a 1-pole low pass filter,
/// temperature compensation and a jumper switch which allows for the selection of 2 sensitivities.
/// Zero-g offset full scale span and filter cut-off are factory set and require no external
/// devices.
///
/// The sensor will measure acceleration in both directions along each of the 3 axis. Acceleration
/// along the X or Y axis in the direction of the silkscreened arrows will produce a larger reading,
/// while acceleration in the opposite direction will produce a smaller reading. For the Z axis,
/// upward acceleration (in the direction of the board's face) produces larger values, and downward
/// acceleration (toward the board's back) produces lower values.
///
/// # Gravity
///
/// Gravity is indistinguishable from upward acceleration, so the sensor will detect a constant 1.0g
/// on the vertical axis while at rest. For example, if the board is mounted horizontally, gravity
/// will effect only the Z axis. If the sensor is tilted away from the horizontal, the gravity
/// reading on the Z axis will diminish, and the readings on the other axis will change depending on
/// the sensor's mounting orientation.
///
/// # Wiring
///
/// Each axis on the accelerometer requires its own ADI port. This means that the accelerometer will
/// take three ADI ports if you wish to measure acceleration on all axes. You don't have to hook up
/// all the channels; you only need to connect the ones required for your application.
///
/// The white (signal) wire of each cable goes near the 'X', 'Y', or 'Z' labels on the board. The
/// black (ground) wires go at the other end, adjacent to the 'B' label on the board. The center
/// wire is for +5 volts. The sensor's mounting holes are electrically isolated from the circuit,
/// meaning it is safe to mount the device using screws on a robot.
///
/// A single axis connection to the 3-axis analog accelerometer.
///
/// The read-only `sensitivity` attribute is the configured `AdiAccelerometerSensitivity`. The
/// read-only `max_acceleration` attribute is the maximum acceleration measurement supported by
/// that physical jumper setting, in g.
#[class(qstr!(AdiAccelerometer))]
#[repr(C)]
pub struct AdiAccelerometerObj {
    base: ObjBase,
    accelerometer: AdiAccelerometer,
}

#[class_methods]
impl AdiAccelerometerObj {
    /// Creates a new accelerometer from `port`.
    ///
    /// `sensitivity` is the physical jumper setting, either `AdiAccelerometerSensitivity.LOW` for a
    /// 0-2g range or `AdiAccelerometerSensitivity.HIGH` for a 0-6g range. `port` is an onboard ADI label
    /// from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If the argument count or either argument's type is invalid.
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(
        sig = "(self, port: str | AdiExpanderPort, sensitivity: AdiAccelerometerSensitivity, /) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        let sensitivity = reader.next_positional::<&SensitivityObj>()?; // TODO: default value?
        let port = port.commit()?;
        Ok(Self {
            base: ty.into(),
            accelerometer: AdiAccelerometer::new(port, sensitivity.sensitivity),
        })
    }

    #[attr]
    #[stub(attrs = ["sensitivity: AdiAccelerometerSensitivity", "max_acceleration: float"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "sensitivity" => Obj::from_static(match self.accelerometer.sensitivity() {
                Sensitivity::Low => SensitivityObj::LOW,
                Sensitivity::High => SensitivityObj::HIGH,
            }),
            "max_acceleration" => Obj::from_float(self.accelerometer.max_acceleration() as f32),
            _ => return,
        })
    }

    /// Returns the current acceleration measurement for this axis in g (~9.8 m/s/s).
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_acceleration(&self) -> Result<f32, Exception> {
        Ok(self.accelerometer.acceleration()? as f32)
    }

    /// Returns the raw acceleration reading from [0, 4095]. This represents an ADC-converted
    /// analog input from 0-5V.
    ///
    /// For example, when on `AdiAccelerometerSensitivity.HIGH` a value of `4095` would represent a
    /// reading of 6g. When on `AdiAccelerometerSensitivity.LOW`, this same value would instead
    /// represent a 2g reading.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn raw_acceleration(&self) -> Result<i32, Exception> {
        Ok(self.accelerometer.raw_acceleration()? as i32)
    }
}
