use argparse::{ArgParser, Args, DefaultParser, ParseError};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::WhiteBalance;

use crate::modvenice::{Exception, read_only_attr::read_only_attr};

#[class(qstr!(WhiteBalance))]
#[repr(C)]
pub struct WhiteBalanceObj {
    base: ObjBase,
}

#[class(qstr!(Auto))]
#[repr(C)]
pub struct Auto {
    base: ObjBase,
}

#[class(qstr!(StartupAuto))]
#[repr(C)]
pub struct StartupAuto {
    base: ObjBase,
}

#[class(qstr!(Manual))]
#[repr(C)]
pub struct Manual {
    base: ObjBase,
    r: u8,
    g: u8,
    b: u8,
}

#[class_methods]
impl WhiteBalanceObj {
    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(
            c"WhiteBalance is an abstract base class; use a variant like WhiteBalance.Signature",
        )
        .raise(token());
    }

    #[constant(qstr!(Auto))]
    const AUTO: &ObjType = Auto::OBJ_TYPE;
    #[constant(qstr!(StartupAuto))]
    const STARTUP_AUTO: &ObjType = StartupAuto::OBJ_TYPE;
    #[constant(qstr!(Manual))]
    const MANUAL: &ObjType = Manual::OBJ_TYPE;
}

#[class_methods]
impl Auto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &'static ObjType, _: usize, _: usize, args: &[Obj]) -> Result<Obj, Exception> {
        if args.len() != 0 {
            Err(
                type_error(c"constructor does not accept arguments; just call WhiteBalance.Auto()")
                    .into(),
            )
        } else {
            Ok(Obj::from_static(Self::SELF))
        }
    }
}

#[class_methods]
impl StartupAuto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &'static ObjType, _: usize, _: usize, args: &[Obj]) -> Result<Obj, Exception> {
        if args.len() != 0 {
            Err(type_error(
                c"constructor does not accept arguments; just call WhiteBalance.StartupAuto()",
            )
            .into())
        } else {
            Ok(Obj::from_static(Self::SELF))
        }
    }
}

#[class_methods]
impl Manual {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(3, 3).assert_nkw(0, 0);

        let r = reader.next_positional::<u8>()?;
        let g = reader.next_positional::<u8>()?;
        let b = reader.next_positional::<u8>()?;

        Ok(Self {
            base: ObjBase::new(ty),
            r,
            g,
            b,
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "r" => self.r,
            "g" => self.g,
            "b" => self.b,
            _ => return,
        } as i32)
    }
}

pub fn new(balance: WhiteBalance) -> Obj {
    match balance {
        WhiteBalance::Auto => Obj::from_static(Auto::SELF),
        WhiteBalance::StartupAuto => Obj::from_static(StartupAuto::SELF),
        WhiteBalance::Manual(color) => Manual {
            base: ObjBase::new(Manual::OBJ_TYPE),
            r: color.r,
            g: color.g,
            b: color.b,
        }
        .into(),
    }
}

#[derive(Default)]
pub struct WhiteBalanceParser;

impl<'a> ArgParser<'a> for WhiteBalanceParser {
    type Output = WhiteBalanceArg;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        if obj.is(Auto::OBJ_TYPE) {
            Ok(WhiteBalanceArg(WhiteBalance::Auto))
        } else if obj.is(StartupAuto::OBJ_TYPE) {
            Ok(WhiteBalanceArg(WhiteBalance::StartupAuto))
        } else if let Some(manual) = obj.try_as_obj::<Manual>() {
            Ok(WhiteBalanceArg(WhiteBalance::Manual(
                (manual.r, manual.g, manual.b).into(),
            )))
        } else {
            Err(ParseError::TypeError {
                expected: "WhiteBalance",
            })
        }
    }
}

pub struct WhiteBalanceArg(pub WhiteBalance);

impl<'a> DefaultParser<'a> for WhiteBalanceArg {
    type Parser = WhiteBalanceParser;
}
