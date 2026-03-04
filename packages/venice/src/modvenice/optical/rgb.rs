use micropython_rs::{
    attr_from_fn,
    except::{mp_type_AttributeError, raise_msg},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::smart::optical::{OpticalRaw, OpticalRgb};

use crate::qstrgen::qstr;

#[repr(C)]
pub struct OpticalRgbObj {
    base: ObjBase<'static>,
    red: f32,
    green: f32,
    blue: f32,
    brightness: f32,
}

#[repr(C)]
pub struct OpticalRawObj {
    base: ObjBase<'static>,
    red: u16,
    green: u16,
    blue: u16,
    clear: u16,
}

pub static OPTICAL_RGB_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(OpticalRgb))
        .set_attr(attr_from_fn!(optical_rgb_attr));

unsafe impl ObjTrait for OpticalRgbObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = OPTICAL_RGB_OBJ_TYPE.as_obj_type();
}

pub static OPTICAL_RAW_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(OpticalRaw))
        .set_attr(attr_from_fn!(optical_raw_attr));

unsafe impl ObjTrait for OpticalRawObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = OPTICAL_RAW_OBJ_TYPE.as_obj_type();
}

impl OpticalRgbObj {
    pub fn new(rgb: OpticalRgb) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            red: rgb.red as f32,
            green: rgb.green as f32,
            blue: rgb.blue as f32,
            brightness: rgb.brightness as f32,
        }
    }
}

impl OpticalRawObj {
    pub fn new(raw: OpticalRaw) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            red: raw.red,
            green: raw.green,
            blue: raw.blue,
            clear: raw.clear,
        }
    }
}

fn optical_rgb_attr(this: &OpticalRgbObj, attr: Qstr, op: AttrOp) {
    let component = match attr.as_str() {
        "red" | "r" => this.red,
        "green" | "g" => this.green,
        "blue" | "b" => this.blue,
        "brightness" => this.brightness,
        _ => return,
    };

    match op {
        AttrOp::Load { result } => result.return_value(Obj::from_float(component)),
        _ => raise_msg(
            token(),
            &mp_type_AttributeError,
            c"cannot write to OpticalRgb",
        ),
    }
}

fn optical_raw_attr(this: &OpticalRawObj, attr: Qstr, op: AttrOp) {
    let component = match attr.as_str() {
        "red" | "r" => this.red,
        "green" | "g" => this.green,
        "blue" | "b" => this.blue,
        "clear" => this.clear,
        _ => return,
    };

    match op {
        AttrOp::Load { result } => result.return_value(Obj::from_int(component as i32)),
        _ => raise_msg(
            token(),
            &mp_type_AttributeError,
            c"cannot write to OpticalRaw",
        ),
    }
}
