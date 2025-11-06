use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::math::Angle;

use crate::qstrgen::qstr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationUnit {
    Radians,
    Degrees,
    Turns,
}

#[repr(C)]
pub struct RotationUnitObj {
    base: ObjBase,
    unit: RotationUnit,
}

pub static ROTATION_UNIT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(RotationUnit)).set_slot_locals_dict_from_static(
        &const_dict![
            qstr!(RADIANS) => Obj::from_static(&RotationUnitObj::RADIANS),
            qstr!(DEGREES) => Obj::from_static(&RotationUnitObj::DEGREES),
            qstr!(TURNS) => Obj::from_static(&RotationUnitObj::TURNS),
        ],
    );

unsafe impl ObjTrait for RotationUnitObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = ROTATION_UNIT_OBJ_TYPE.as_obj_type();
}

impl RotationUnit {
    pub fn in_angle(self, angle: Angle) -> f32 {
        (match self {
            Self::Radians => angle.as_radians(),
            Self::Degrees => angle.as_degrees(),
            Self::Turns => angle.as_turns(),
        }) as f32
    }
}

impl RotationUnitObj {
    pub const RADIANS: Self = Self::new(RotationUnit::Radians);
    pub const DEGREES: Self = Self::new(RotationUnit::Degrees);
    pub const TURNS: Self = Self::new(RotationUnit::Turns);

    pub const fn new(unit: RotationUnit) -> Self {
        Self {
            base: ObjBase::new(ROTATION_UNIT_OBJ_TYPE.as_obj_type()),
            unit,
        }
    }

    pub const fn unit(&self) -> RotationUnit {
        self.unit
    }
}
