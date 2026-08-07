use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::adi::potentiometer::{AdiPotentiometer, PotentiometerType};

use crate::modvenice::{
    Exception, adi::expander::AdiPortParser, read_only_attr::read_only_attr,
    units::rotation::RotationUnitObj,
};

/// ADI Potentiometer
///
/// This class provides an interface for interacting with VEX's ADI potentiometers.
///
/// # Hardware Overview
///
/// Potentiometers are analog sensors that measure angular position. They function as variable
/// resistors that change their resistance based on the angular position of their shaft.
///
/// VEX offers two variants:
///
/// - Legacy (EDR) Potentiometer: Provides measurements across a 250-degree range.
/// - V2 Potentiometer: Provides measurements across a 333-degree range.
///
/// Both variants connect to the ADI ports and provide analog signals that are converted to
/// measurements of a shaft's angle.
///
/// # Comparison to `AdiEncoder`
///
/// Potentiometers are fundamentally *analog* sensors. They directly output a measurement of their
/// electrical resistance to the ADI port. The more a shaft rotates along a conductive material
/// inside of them, the higher the reported angle.
///
/// With this in mind, this means that potentiometers are capable of measuring absolute position at
/// *all times*, even after they have lost power. Encoders on the other hand can only track *changes
/// in position* as a digital signal, meaning that any changes in rotation under an encoder can only
/// be recorded while the encoder is plugged in and being read.
///
/// # Comparison to `RotationSensor`
///
/// Rotation sensors operate similarly to a potentiometer, in that they know their absolute angle at
/// all times (even when being powered off). This is achieved through a hall-effect sensor rather
/// than a conductive material, however. Rotation sensors can also measure their position along with
/// their angle, similar to how an encoder can. They also have a full range of motion and can track
/// angle/position in a full 360-degree range. Potentiometers use ADI ports while Rotation Sensors
/// use Smart ports.
///
/// Potentiometer
///
/// The read-only `type` attribute is the configured `PotentiometerType`.
#[class(qstr!(AdiPotentiometer))]
#[repr(C)]
pub struct AdiPotentiometerObj {
    base: ObjBase,
    potentiometer: AdiPotentiometer,
}

/// The type of potentiometer device.
#[class(qstr!(PotentiometerType))]
#[repr(C)]
pub struct PotentiometerTypeObj {
    base: ObjBase,
    ty: PotentiometerType,
}

#[class_methods]
impl PotentiometerTypeObj {
    const fn new(ty: PotentiometerType) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            ty,
        }
    }

    /// EDR potentiometer.
    #[constant]
    const LEGACY: &Self = &Self::new(PotentiometerType::Legacy);
    /// V2 potentiometer.
    #[constant]
    const V2: &Self = &Self::new(PotentiometerType::V2);

    /// Maximum angle in degrees for the older cortex-era EDR potentiometer.
    #[constant]
    const LEGACY_MAX_ANGLE_DEG: f32 = 250.0;
    /// Maximum angle in degrees for the V5-era potentiometer V2.
    #[constant]
    const V2_MAX_ANGLE_DEG: f32 = 333.0;

    /// Returns the maximum angle measurement in the supplied `unit` for this potentiometer type.
    #[method]
    fn get_max_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(self.ty.max_angle())
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.ty {
            PotentiometerType::Legacy => "PotentiometerType.LEGACY",
            PotentiometerType::V2 => "PotentiometerType.V2",
        });
    }
}

#[class_methods]
impl AdiPotentiometerObj {
    /// Creates a new potentiometer from `port`.
    ///
    /// `potentiometer_type` selects either the legacy EDR potentiometer or the V5-era potentiometer V2.
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     potentiometer = AdiPotentiometer("A", PotentiometerType.V2)
    ///     while True:
    ///         angle = potentiometer.get_angle(DEGREES)
    ///         print("Potentiometer Angle:", angle)
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(
        sig = "(self, port: str | AdiExpanderPort, potentiometer_type: PotentiometerType) -> None"
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
        let potentiometer_type = reader.next_positional::<&PotentiometerTypeObj>()?;

        Ok(Self {
            base: ty.into(),
            potentiometer: AdiPotentiometer::new(port, potentiometer_type.ty),
        })
    }

    #[attr]
    #[stub(attrs = ["type: PotentiometerType"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "type" => Obj::from_static(match self.potentiometer.potentiometer_type() {
                PotentiometerType::Legacy => PotentiometerTypeObj::LEGACY,
                PotentiometerType::V2 => PotentiometerTypeObj::V2,
            }),
            _ => return,
        })
    }

    /// Returns the maximum angle measurement in the supplied `unit` for the configured `PotentiometerType`.
    #[method]
    fn get_max_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(self.potentiometer.max_angle())
    }

    /// Returns the current potentiometer angle in the supplied `unit`.
    ///
    /// The original potentiometer rotates 250 degrees thus returning an angle between 0-250
    /// degrees. Potentiometer V2 rotates 333 degrees thus returning an angle between 0-333 degrees.
    ///
    /// # Example
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     potentiometer = AdiPotentiometer("A", PotentiometerType.V2)
    ///     while True:
    ///         angle = potentiometer.get_angle(DEGREES)
    ///         print("Potentiometer Angle:", angle)
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.potentiometer.angle()?))
    }
}
