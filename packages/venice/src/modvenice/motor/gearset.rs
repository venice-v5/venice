use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::motor::Gearset;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct GearsetObj {
    base: ObjBase<'static>,
    gearset: Gearset,
}

static GEARSET_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_locals_dict(const_dict![
        qstr!(RED) => Obj::from_static(&GearsetObj::RED),
        qstr!(GREEN) => Obj::from_static(&GearsetObj::GREEN),
        qstr!(BLUE) => Obj::from_static(&GearsetObj::BLUE),
    ]);

unsafe impl ObjTrait for GearsetObj {
    const OBJ_TYPE: &ObjType = GEARSET_OBJ_TYPE.as_obj_type();
}

impl GearsetObj {
    pub const RED: Self = Self::new(Gearset::Red);
    pub const GREEN: Self = Self::new(Gearset::Green);
    pub const BLUE: Self = Self::new(Gearset::Blue);

    pub const fn new(gearset: Gearset) -> Self {
        Self {
            base: ObjBase::new(GEARSET_OBJ_TYPE.as_obj_type()),
            gearset,
        }
    }

    pub const fn new_static(gearset: Gearset) -> &'static Self {
        match gearset {
            Gearset::Red => &Self::RED,
            Gearset::Green => &Self::GREEN,
            Gearset::Blue => &Self::BLUE,
        }
    }

    pub const fn gearset(&self) -> Gearset {
        self.gearset
    }
}
