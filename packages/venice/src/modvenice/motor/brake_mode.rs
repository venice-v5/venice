use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::{math::Direction, smart::motor::BrakeMode};

use crate::qstrgen::qstr;

static BRAKE_MODE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Direction))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(COAST) => Obj::from_static(&BrakeModeObj::COAST),
        qstr!(BRAKE) => Obj::from_static(&BrakeModeObj::BRAKE),
        qstr!(HOLD) => Obj::from_static(&BrakeModeObj::HOLD),
    ]);

#[repr(C)]
pub struct BrakeModeObj {
    base: ObjBase,
    mode: BrakeMode,
}

unsafe impl ObjTrait for BrakeModeObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = BRAKE_MODE_OBJ_TYPE.as_obj_type();
}

impl BrakeModeObj {
    pub const COAST: Self = Self::new(BrakeMode::Coast);
    pub const BRAKE: Self = Self::new(BrakeMode::Brake);
    pub const HOLD: Self = Self::new(BrakeMode::Hold);

    pub const fn new(mode: BrakeMode) -> Self {
        Self {
            base: ObjBase::new(BRAKE_MODE_OBJ_TYPE.as_obj_type()),
            mode,
        }
    }

    pub const fn mode(&self) -> BrakeMode {
        self.mode
    }
}
