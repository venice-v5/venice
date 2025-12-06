use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::motor::MotorType;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct MotorTypeObj {
    base: ObjBase<'static>,
    motor_type: MotorType,
}

static MOTOR_TYPE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_locals_dict(const_dict![
        qstr!(EXP) => Obj::from_static(&MotorTypeObj::EXP),
        qstr!(V5) => Obj::from_static(&MotorTypeObj::V5),
    ]);

unsafe impl ObjTrait for MotorTypeObj {
    const OBJ_TYPE: &ObjType = MOTOR_TYPE_OBJ_TYPE.as_obj_type();
}

impl MotorTypeObj {
    pub const V5: Self = Self::new(MotorType::V5);
    pub const EXP: Self = Self::new(MotorType::Exp);

    pub const fn new(motor_type: MotorType) -> Self {
        Self {
            base: ObjBase::new(MOTOR_TYPE_OBJ_TYPE.as_obj_type()),
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
