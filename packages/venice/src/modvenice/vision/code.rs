use argparse::Args;
use micropython_rs::{
    class, class_methods,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::VisionCode;

use crate::modvenice::vision::validate_id;

#[class(qstr!(VisionCode))]
#[repr(C)]
pub struct VisionCodeObj {
    base: ObjBase<'static>,
    code: VisionCode,
}

#[class_methods]
impl VisionCodeObj {
    pub fn new(code: VisionCode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            code,
        }
    }

    pub fn code(&self) -> VisionCode {
        self.code
    }

    #[make_new]
    fn make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Self {
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());

        let s1 = validate_id(reader.next_positional());
        let s2 = validate_id(reader.next_positional());

        let s3 = reader.try_next_positional().map(validate_id).ok();
        let s4 = reader.try_next_positional().map(validate_id).ok();
        let s5 = reader.try_next_positional().map(validate_id).ok();

        Self {
            base: ObjBase::new(ty),
            code: VisionCode::new(s1, s2, s3, s4, s5),
        }
    }

    #[method(binding = "static")]
    fn from_id(id: i32) -> Self {
        Self::new(VisionCode::from_id(id as u16))
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "sig1" => Obj::from(self.code.0 as i32),
            "sig2" => (self.code.1 as i32).into(),
            "sig3" => self.code.2.map(i32::from).into(),
            "sig4" => self.code.3.map(i32::from).into(),
            "sig5" => self.code.4.map(i32::from).into(),
            _ => return,
        })
    }
}
