use micropython_macros::{class, class_methods};
use micropython_rs::obj::{ObjBase, ObjTrait};
use vexide_devices::smart::motor::MotorType;

#[class(qstr!(MotorType))]
#[repr(C)]
pub struct MotorTypeObj {
    base: ObjBase,
    motor_type: MotorType,
}

#[class_methods]
impl MotorTypeObj {
    #[constant]
    pub const V5: &Self = &Self::new(MotorType::V5);
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
}
