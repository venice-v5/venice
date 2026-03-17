use argparse::{Args, Exception};
use micropython_rs::{
    class, class_methods,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use rgb::Rgb;
use vexide_devices::smart::ai_vision::AiVisionColor;

#[class(qstr!(AiVisionColor))]
#[repr(C)]
pub struct AiVisionColorObj {
    base: ObjBase<'static>,
    color: AiVisionColor,
}

impl AiVisionColorObj {
    pub fn color(&self) -> AiVisionColor {
        self.color
    }

    pub fn new(color: AiVisionColor) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            color,
        }
    }
}

#[class_methods]
impl AiVisionColorObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "r" => Obj::from_int(self.color.rgb.r as _),
            "g" => Obj::from_int(self.color.rgb.g as _),
            "b" => Obj::from_int(self.color.rgb.b as _),
            "hue_range" => Obj::from_float(self.color.hue_range as _),
            "saturation_range" => Obj::from_float(self.color.saturation_range as _),
            _ => return,
        });
    }

    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let token = token();
        let mut reader = Args::new(n_pos, n_kw, args).reader(token);
        reader.assert_npos(5, 5).assert_nkw(0, 0);
        let rgb = Rgb::<u8>::new(
            reader.next_positional::<u8>()?,
            reader.next_positional::<u8>()?,
            reader.next_positional::<u8>()?,
        );
        let color = AiVisionColor {
            rgb,
            hue_range: reader.next_positional::<f32>()?,
            saturation_range: reader.next_positional::<f32>()?,
        };
        Ok(Self {
            base: ObjBase::new(ty),
            color,
        })
    }
}
