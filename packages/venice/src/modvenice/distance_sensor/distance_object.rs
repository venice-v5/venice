use micropython_rs::{obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags}, qstr::Qstr};
use vexide_devices::smart::distance::DistanceObject;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct DistanceObjectObj {
    base: ObjBase<'static>,
    object: DistanceObject,
}

pub static DISTANCE_OBJECT_OBJ_TYPE: ObjFullType = unsafe {
    ObjFullType::new(TypeFlags::empty(), qstr!(DistanceObject))
        .set_slot_attr(distance_object_attr)
};

unsafe impl ObjTrait for DistanceObjectObj {
    const OBJ_TYPE: &ObjType = DISTANCE_OBJECT_OBJ_TYPE.as_obj_type();
}

impl DistanceObjectObj {
    pub fn new(object: DistanceObject) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            object,
        }
    }
}

unsafe extern "C" fn distance_object_attr(self_in: Obj, attr: Qstr, dest: *mut Obj) {
    if unsafe { *dest }.is_sentinel() {
        return;
    }

    let state = &self_in.try_to_obj::<DistanceObjectObj>().unwrap().object;
    let attr_bytes = attr.bytes();
    let field_obj = match attr_bytes {
        b"confidence" => Obj::from_float(state.confidence as _),
        b"distance" => Obj::from_int(state.distance as _),
        b"relative_size" => Obj::from_int(state.relative_size as _),
        b"velocity" => Obj::from_float(state.velocity as _),
        _ => return
    };

    unsafe { *dest = field_obj };
}
