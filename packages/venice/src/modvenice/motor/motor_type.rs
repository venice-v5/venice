use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::motor::MotorType;

/// Represents the type of a Smart Motor: either an 11W (V5) or 5.5W (EXP) motor.
///
/// Values are returned by the read-only
/// `Motor.motor_type` attribute. `MotorType` is root-importable and isn't constructed directly;
/// use its singleton constants. Values have readable representations such as `MotorType.V5`.
#[class(qstr!(MotorType))]
#[repr(C)]
pub struct MotorTypeObj {
    base: ObjBase,
    motor_type: MotorType,
}

#[class_methods]
impl MotorTypeObj {
    /// An 11W Smart Motor.
    #[constant]
    pub const V5: &Self = &Self::new(MotorType::V5);
    /// A 5.5W Smart Motor.
    #[constant]
    pub const EXP: &Self = &Self::new(MotorType::Exp);

    pub const fn new(motor_type: MotorType) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            motor_type,
        }
    }

    pub const fn new_static(motor_type: MotorType) -> &'static Self {
        match motor_type {
            MotorType::V5 => &MotorTypeObj::V5,
            MotorType::Exp => &MotorTypeObj::EXP,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.motor_type {
            MotorType::V5 => "MotorType.V5",
            MotorType::Exp => "MotorType.EXP",
        })
    }
}
