use argparse::{ArgType, Args, PositionalError};
use micropython_rs::{
    class, class_methods,
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::DetectionSource;

use crate::modvenice::{
    Exception,
    vision::{SignatureId, code::VisionCodeObj},
};

#[class(qstr!(DetectionSource))]
#[repr(C)]
pub struct DetectionSourceObj {
    base: ObjBase<'static>,
}

#[class(qstr!(Signature))]
#[repr(C)]
pub struct Signature {
    base: ObjBase<'static>,
    id: u8,
}

#[class(qstr!(Code))]
#[repr(C)]
pub struct Code {
    base: ObjBase<'static>,
    code: Obj,
}

#[class(qstr!(Line))]
#[repr(C)]
pub struct Line {
    base: ObjBase<'static>,
}

#[class_methods]
impl DetectionSourceObj {
    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(c"DetectionSource is an abstract base class; use a variant like DetectionSource.Signature").raise(token());
    }

    #[constant(qstr!(Signature))]
    const SIGNATURE: &ObjType = Signature::OBJ_TYPE;
    #[constant(qstr!(Code))]
    const CODE: &ObjType = Code::OBJ_TYPE;
    #[constant(qstr!(Line))]
    const LINE: &ObjType = Line::OBJ_TYPE;
}

#[class_methods]
impl Signature {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let id = reader.next_positional::<SignatureId>()?.id();

        Ok(Self {
            base: ObjBase::new(ty),
            id,
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "id" => self.id as i32,
            _ => return,
        });
    }
}

#[class_methods]
impl Code {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let code_obj = reader.next_positional::<Obj>().unwrap();
        if code_obj.is(VisionCodeObj::OBJ_TYPE) {
            Ok(Self {
                base: ObjBase::new(ty),
                code: code_obj,
            })
        } else {
            Err(PositionalError::TypeError {
                n: 1,
                expected: "VisionCode",
                found: &format!("{}", ArgType::of(&code_obj)),
            }
            .into())
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "code" => self.code,
            _ => return,
        });
    }
}

#[class_methods]
impl Line {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, args: &[Obj]) -> Result<Obj, Exception> {
        if args.len() != 0 {
            Err(type_error(
                c"constructor does not accept arguments; just call DetectionSource.Line()",
            )
            .into())
        } else {
            Ok(Obj::from_static(Self::SELF))
        }
    }
}

pub fn new(source: DetectionSource) -> Obj {
    match source {
        DetectionSource::Signature(id) => Signature {
            base: ObjBase::new(Signature::OBJ_TYPE),
            id,
        }
        .into(),
        DetectionSource::Code(code) => Code {
            base: ObjBase::new(Code::OBJ_TYPE),
            code: super::code::VisionCodeObj::new(code).into(),
        }
        .into(),
        DetectionSource::Line => Obj::from_static(Line::SELF),
    }
}
