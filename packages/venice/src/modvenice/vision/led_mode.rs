use argparse::{ArgParser, Args, DefaultParser, ParseError};
use micropython_rs::{
    class, class_methods,
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::LedMode;

use crate::{modvenice::Exception, obj::alloc_obj};

#[class(qstr!(LedMode))]
pub struct LedModeObj {
    base: ObjBase<'static>,
}

#[class(qstr!(Auto))]
pub struct Auto {
    base: ObjBase<'static>,
}

#[class(qstr!(Manual))]
pub struct Manual {
    base: ObjBase<'static>,
    brightness: f32,
    r: u8,
    g: u8,
    b: u8,
}

#[class_methods]
impl LedModeObj {
    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(c"LedMode is an abstract base class; use a variant like LedMode.Auto")
            .raise(token())
    }

    #[constant(qstr!(Auto))]
    const AUTO: &ObjType = Auto::OBJ_TYPE;
    #[constant(qstr!(Manual))]
    const MANUAL: &ObjType = Manual::OBJ_TYPE;
}

#[class_methods]
impl Auto {
    #[parent]
    const PARENT: &ObjType = LedModeObj::OBJ_TYPE;

    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, args: &[Obj]) -> Result<Obj, Exception> {
        if args.len() != 0 {
            Err(
                type_error(c"constructor does not accept arguments; just call LedMode.Auto()")
                    .into(),
            )
        } else {
            Ok(Obj::from_static(Self::SELF))
        }
    }
}

#[class_methods]
impl Manual {
    #[parent]
    const PARENT: &ObjType = LedModeObj::OBJ_TYPE;

    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(4, 4).assert_nkw(0, 0);

        let brightness = reader.next_positional()?;
        let r = reader.next_positional()?;
        let g = reader.next_positional()?;
        let b = reader.next_positional()?;

        Ok(Self {
            base: ObjBase::new(ty),
            brightness,
            r,
            g,
            b,
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "brightness" => Obj::from(self.brightness),
            "r" => (self.r as i32).into(),
            "g" => (self.g as i32).into(),
            "b" => (self.b as i32).into(),
            _ => return,
        })
    }

    pub fn as_led_mode(&self) -> LedMode {
        LedMode::Manual((self.r, self.g, self.b).into(), self.brightness as f64)
    }
}

pub fn new(mode: LedMode) -> Obj {
    match mode {
        LedMode::Auto => Obj::from_static(Auto::SELF),
        LedMode::Manual(color, brightness) => alloc_obj(Manual {
            base: ObjBase::new(Manual::OBJ_TYPE),
            brightness: brightness as f32,
            r: color.r,
            g: color.g,
            b: color.b,
        }),
    }
}

#[derive(Default)]
pub struct LedModeParser;
pub struct LedModeArg(pub LedMode);

impl<'a> ArgParser<'a> for LedModeParser {
    type Output = LedModeArg;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        if obj.is(Auto::OBJ_TYPE) {
            Ok(LedModeArg(LedMode::Auto))
        } else if let Some(manual) = obj.try_as_obj::<Manual>() {
            Ok(LedModeArg(manual.as_led_mode()))
        } else {
            Err(ParseError::TypeError {
                expected: "LedMode",
            })
        }
    }
}

impl DefaultParser<'_> for LedModeArg {
    type Parser = LedModeParser;
}
