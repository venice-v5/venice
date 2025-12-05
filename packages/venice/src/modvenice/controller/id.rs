use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::controller::ControllerId;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct ControllerIdObj {
    base: ObjBase<'static>,
    id: ControllerId,
}

pub static CONTROLLER_ID_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(ControllerId)).set_slot_locals_dict_from_static(
        &const_dict![
            qstr!(PRIMARY) => Obj::from_static(&ControllerIdObj::PRIMARY),
            qstr!(PARTNER) => Obj::from_static(&ControllerIdObj::PARTNER),
        ],
    );

unsafe impl ObjTrait for ControllerIdObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = CONTROLLER_ID_OBJ_TYPE.as_obj_type();
}

impl ControllerIdObj {
    pub const PRIMARY: Self = Self::new(ControllerId::Primary);
    pub const PARTNER: Self = Self::new(ControllerId::Partner);

    pub const fn new(id: ControllerId) -> Self {
        Self {
            base: ObjBase::new(CONTROLLER_ID_OBJ_TYPE.as_obj_type()),
            id,
        }
    }

    pub const fn id(&self) -> ControllerId {
        self.id
    }
}
