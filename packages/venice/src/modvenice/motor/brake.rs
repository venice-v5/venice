use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::smart::motor::BrakeMode;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct BrakeModeObj {
    base: ObjBase<'static>,
    mode: BrakeMode,
}

pub static BRAKE_MODE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(BrakeMode)).set_slot_locals_dict_from_static(
        &const_dict![
            qstr!(BRAKE) => Obj::from_static(&BrakeModeObj::BRAKE),
            qstr!(COAST) => Obj::from_static(&BrakeModeObj::COAST),
            qstr!(HOLD) => Obj::from_static(&BrakeModeObj::HOLD),
        ],
    );

unsafe impl ObjTrait for BrakeModeObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = BRAKE_MODE_OBJ_TYPE.as_obj_type();
}

impl BrakeModeObj {
    pub const BRAKE: Self = Self::new(BrakeMode::Brake);
    pub const COAST: Self = Self::new(BrakeMode::Coast);
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
