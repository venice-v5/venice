use micropython_rs::{
    attr_from_fn, const_dict,
    map::Dict,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::smart::ai_vision::AiVisionColor;

use crate::{fun::fun2_from_fn, modvenice::units::rotation::RotationUnitObj, obj::alloc_obj, qstrgen::qstr};

static AI_VISION_COLOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionColor)).set_attr(attr_from_fn!(ai_vision_color_state_attr));

#[repr(C)]
pub struct AiVisionColorObj {
    base: ObjBase<'static>,
    color: AiVisionColor,
}

unsafe impl ObjTrait for AiVisionColorObj {
    const OBJ_TYPE: &ObjType = AI_VISION_COLOR_OBJ_TYPE.as_obj_type();
}

fn ai_vision_color_state_attr(this: &AiVisionColorObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { dest } = op else { return };
    *dest = match attr.bytes() {
        b"r" => Obj::from_int(this.color.rgb.r as _),
        b"g" => Obj::from_int(this.color.rgb.g as _),
        b"b" => Obj::from_int(this.color.rgb.b as _),
        b"hue_range" => Obj::from_float(this.color.hue_range as _),
        b"saturation_range" => Obj::from_float(this.color.saturation_range as _),
        _ => return
    }
}
