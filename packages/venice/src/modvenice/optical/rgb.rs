use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{ATTRIBUTE_ERROR_TYPE, raise_msg},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::smart::optical::{OpticalRaw, OpticalRgb};

#[class(qstr!(OpticalRgb))]
#[repr(C)]
pub struct OpticalRgbObj {
    base: ObjBase,
    red: f32,
    green: f32,
    blue: f32,
    brightness: f32,
}

#[class(qstr!(OpticalRaw))]
#[repr(C)]
pub struct OpticalRawObj {
    base: ObjBase,
    red: u16,
    green: u16,
    blue: u16,
    clear: u16,
}

#[class_methods]
impl OpticalRgbObj {
    pub fn new(rgb: OpticalRgb) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            red: rgb.red as f32,
            green: rgb.green as f32,
            blue: rgb.blue as f32,
            brightness: rgb.brightness as f32,
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let component = match attr.as_str() {
            "red" | "r" => self.red,
            "green" | "g" => self.green,
            "blue" | "b" => self.blue,
            "brightness" => self.brightness,
            _ => return,
        };

        match op {
            AttrOp::Load { result } => result.return_value(Obj::from_float(component)),
            _ => raise_msg(token(), ATTRIBUTE_ERROR_TYPE, c"cannot write to OpticalRgb"),
        }
    }
}

#[class_methods]
impl OpticalRawObj {
    pub fn new(raw: OpticalRaw) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            red: raw.red,
            green: raw.green,
            blue: raw.blue,
            clear: raw.clear,
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let component = match attr.as_str() {
            "red" | "r" => self.red,
            "green" | "g" => self.green,
            "blue" | "b" => self.blue,
            "clear" => self.clear,
            _ => return,
        };

        match op {
            AttrOp::Load { result } => result.return_value(Obj::from_int(component as i32)),
            _ => raise_msg(token(), ATTRIBUTE_ERROR_TYPE, c"cannot write to OpticalRaw"),
        }
    }
}
