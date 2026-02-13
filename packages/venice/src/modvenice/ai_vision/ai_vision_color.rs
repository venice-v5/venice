use micropython_rs::{
    attr_from_fn,
    init::token,
    make_new_from_fn,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
};
use rgb::Rgb;
use vexide_devices::smart::ai_vision::AiVisionColor;

use crate::{args::Args, obj::alloc_obj, qstrgen::qstr};

static AI_VISION_COLOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionColor))
        .set_attr(attr_from_fn!(ai_vision_color_state_attr))
        .set_make_new(make_new_from_fn!(ai_vision_color_make_new));

#[repr(C)]
pub struct AiVisionColorObj {
    base: ObjBase<'static>,
    color: AiVisionColor,
}

unsafe impl ObjTrait for AiVisionColorObj {
    const OBJ_TYPE: &ObjType = AI_VISION_COLOR_OBJ_TYPE.as_obj_type();
}

impl AiVisionColorObj {
    pub fn color(&self) -> AiVisionColor {
        self.color
    }
    pub fn new(color: AiVisionColor) -> Self {
        Self {
            base: ObjBase::new(AI_VISION_COLOR_OBJ_TYPE.as_obj_type()),
            color,
        }
    }
}

fn ai_vision_color_state_attr(this: &AiVisionColorObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else { return };
    result.return_value(match attr.bytes() {
        b"r" => Obj::from_int(this.color.rgb.r as _),
        b"g" => Obj::from_int(this.color.rgb.g as _),
        b"b" => Obj::from_int(this.color.rgb.b as _),
        b"hue_range" => Obj::from_float(this.color.hue_range as _),
        b"saturation_range" => Obj::from_float(this.color.saturation_range as _),
        _ => return,
    });
}

fn ai_vision_color_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(5, 5);
    let rgb = Rgb::<u8>::new(
        reader.next_positional::<i32>() as _,
        reader.next_positional::<i32>() as _,
        reader.next_positional::<i32>() as _,
    );
    let color = AiVisionColor {
        rgb,
        hue_range: reader.next_positional::<f32>(),
        saturation_range: reader.next_positional::<f32>(),
    };
    alloc_obj(AiVisionColorObj {
        base: ObjBase::new(ty),
        color,
    })
}
