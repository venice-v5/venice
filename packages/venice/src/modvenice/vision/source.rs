use std::fmt::Write;

use argparse::{ArgType, Args, PositionalError};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::smart::vision::{DetectionSource, VisionCode};

use crate::modvenice::{
    Exception,
    read_only_attr::read_only_attr,
    vision::{SignatureId, code::VisionCodeObj},
};

/// The detection method used to identify a `VisionObject`.
///
/// `DetectionSource` is an abstract base and cannot be instantiated directly. Construct its
/// associated-only variants as `DetectionSource.Signature(...)`, `DetectionSource.Code(...)`, or
/// `DetectionSource.Line()`. The variant classes are not package-root imports.
#[class(qstr!(DetectionSource))]
#[repr(C)]
pub struct DetectionSourceObj {
    base: ObjBase,
}

/// A normal Vision signature not associated with a color code was used to detect a `VisionObject`.
///
/// Construct this associated-only class as `DetectionSource.Signature(id)`; it is not package-root
/// importable. The read-only integer attribute `id` is the matching signature slot from 1 to 7. Values
/// compare equal when their IDs match and print as `DetectionSource.Signature(id=...)`.
#[class(qstr!(Signature))]
#[repr(C)]
pub struct Signature {
    base: ObjBase,
    id: u8,
}

/// Multiple signatures joined in a color code were used to detect a `VisionObject`.
///
/// Construct this associated-only class as `DetectionSource.Code(code)`; it is not package-root
/// importable. The read-only `code` attribute is the matching `VisionCode`. Values compare equal when
/// their codes match and print as `DetectionSource.Code(code=...)`.
#[class(qstr!(Code))]
#[repr(C)]
pub struct Code {
    base: ObjBase,
    code: Obj,
}

/// Line detection was used to find a `VisionObject`.
///
/// Construct this associated-only class as `DetectionSource.Line()`; it is not package-root
/// importable. Construction always returns the same singleton, which prints as
/// `DetectionSource.Line()`.
#[class(qstr!(Line))]
#[repr(C)]
pub struct Line {
    base: ObjBase,
}

#[class_methods]
impl DetectionSourceObj {
    /// Rejects direct construction of the abstract `DetectionSource` base class.
    ///
    /// Use one of the associated variant classes instead.
    ///
    /// # Raises
    ///
    /// - `TypeError`: Always.
    #[make_new]
    #[stub(sig = "(self) -> None")]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(c"DetectionSource is an abstract base class; use a variant like DetectionSource.Signature").raise(token());
    }

    /// The associated class for a normal Vision signature source, constructed as `DetectionSource.Signature(id)`.
    #[constant(qstr!(Signature))]
    const SIGNATURE: &ObjType = Signature::OBJ_TYPE;
    /// The associated class for multiple signatures joined in a color code, constructed as `DetectionSource.Code(code)`.
    #[constant(qstr!(Code))]
    const CODE: &ObjType = Code::OBJ_TYPE;
    /// The associated class for line detection, constructed as `DetectionSource.Line()`.
    #[constant(qstr!(Line))]
    const LINE: &ObjType = Line::OBJ_TYPE;
}

#[class_methods]
impl Signature {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    /// Creates a normal Vision signature source for signature slot `id`.
    ///
    /// `id` is a positional-only integer from 1 to 7.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `id` is not an integer, a keyword argument is supplied, or the argument count is
    ///   not one.
    /// - `ValueError`: If `id` is outside the inclusive range 1 to 7.
    #[make_new]
    #[stub(sig = "(self, id: int) -> None")]
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
    #[stub(attrs = ["id: int"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "id" => self.id as i32,
            _ => return,
        });
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.id == rhs.as_obj::<Self>().id),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(print, "DetectionSource.Signature(id={})", self.id);
    }
}

#[class_methods]
impl Code {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    /// Creates a source indicating that multiple signatures joined in `code` detected the object.
    ///
    /// `code` must be a positional-only `VisionCode`.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `code` is not a `VisionCode`, a keyword argument is supplied, or the argument
    ///   count is not one.
    #[make_new]
    #[stub(sig = "(self, code: VisionCode) -> None")]
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

    fn code(&self) -> VisionCode {
        self.code.as_obj::<VisionCodeObj>().code()
    }

    #[attr]
    #[stub(attrs = ["code: VisionCode"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "code" => self.code,
            _ => return,
        });
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.code() == rhs.as_obj::<Self>().code()),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, kind: PrintKind) {
        print.print("DetectionSource.Code(code=");
        let _ = self.code.print(print, kind);
        print.print(")");
    }
}

#[class_methods]
impl Line {
    #[parent]
    const PARENT: &ObjType = DetectionSourceObj::OBJ_TYPE;

    const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    /// Returns the singleton source indicating that line detection found the object.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If any positional or keyword arguments are supplied.
    #[make_new]
    #[stub(sig = "(self) -> None")]
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("DetectionSource.Line()");
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
