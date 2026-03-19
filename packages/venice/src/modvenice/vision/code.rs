use argparse::{Args, ArgsReader, PositionalError};
use micropython_rs::{
    class, class_methods,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::VisionCode;

use crate::modvenice::{Exception, vision::SignatureId};

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
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 5).assert_nkw(0, 0);

        let s1 = reader.next_positional::<SignatureId>()?.id();
        let s2 = reader.next_positional::<SignatureId>()?.id();

        fn read_optional_sig(reader: &mut ArgsReader) -> Result<Option<u8>, Exception> {
            let result = reader.next_positional::<SignatureId>();
            match result {
                Ok(id) => Ok(Some(id.id())),
                Err(PositionalError::ArgumentsExhausted) => Ok(None),
                _ => Err(Exception::from(result.unwrap_err())),
            }
        }

        let s3 = read_optional_sig(&mut reader)?;
        let s4 = read_optional_sig(&mut reader)?;
        let s5 = read_optional_sig(&mut reader)?;

        Ok(Self {
            base: ObjBase::new(ty),
            code: VisionCode::new(s1, s2, s3, s4, s5),
        })
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
