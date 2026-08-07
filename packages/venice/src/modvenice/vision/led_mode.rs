use std::fmt::Write;

use argparse::{ArgParser, Args, DefaultParser, ParseError};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::smart::vision::LedMode;

use crate::{
    modvenice::{Exception, read_only_attr::read_only_attr},
    obj::alloc_obj,
};

/// Vision Sensor LED mode.
///
/// Represents the states that the integrated LED indicator on a Vision Sensor can be in. `LedMode` is
/// an abstract base and cannot be instantiated directly. Construct its associated-only variants as
/// `LedMode.Auto()` or `LedMode.Manual(...)`; the variants are not package-root imports.
#[class(qstr!(LedMode))]
pub struct LedModeObj {
    base: ObjBase,
}

/// Automatic Mode.
///
/// When in automatic mode, the integrated LED will display the color of the most prominent detected
/// object's signature color. This associated-only class is constructed as `LedMode.Auto()` and is not
/// package-root importable. Construction always returns the same singleton, which prints as
/// `LedMode.Auto()`.
#[class(qstr!(Auto))]
pub struct Auto {
    base: ObjBase,
}

/// Manual Mode.
///
/// When in manual mode, the integrated LED will display a user-set RGB color code and brightness
/// percentage from 0.0-1.0. This associated-only class is constructed as `LedMode.Manual(...)` and is
/// not package-root importable. Its read-only `r`, `g`, and `b` attributes are RGB channels from 0 to
/// 255; its read-only `brightness` attribute is the intended normalized brightness. Values compare by
/// these four attributes and print as `LedMode.Manual(r=..., g=..., b=..., brightness=...)`.
#[class(qstr!(Manual))]
pub struct Manual {
    base: ObjBase,
    brightness: f32,
    r: u8,
    g: u8,
    b: u8,
}

#[class_methods]
impl LedModeObj {
    /// Rejects direct construction of the abstract `LedMode` base class.
    ///
    /// Use `LedMode.Auto()` or `LedMode.Manual(r, g, b, brightness)` instead.
    ///
    /// # Raises
    ///
    /// - `TypeError`: Always.
    #[make_new]
    #[stub(sig = "(self) -> None")]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(c"LedMode is an abstract base class; use a variant like LedMode.Auto")
            .raise(token())
    }

    /// The associated Automatic Mode class, constructed as `LedMode.Auto()`.
    #[constant(qstr!(Auto))]
    const AUTO: &ObjType = Auto::OBJ_TYPE;
    /// The associated Manual Mode class, constructed as `LedMode.Manual(...)`.
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

    /// Returns the singleton Automatic Mode.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If any positional or keyword arguments are supplied.
    #[make_new]
    #[stub(sig = "(self) -> None")]
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("LedMode.Auto()");
    }
}

#[class_methods]
impl Manual {
    #[parent]
    const PARENT: &ObjType = LedModeObj::OBJ_TYPE;

    /// Creates a Manual Mode with RGB channels `r`, `g`, and `b` and normalized `brightness`.
    ///
    /// RGB channels are positional-only integers from 0 to 255. `brightness` is positional-only, accepts
    /// an integer or float, and is intended to range from 0.0 for off to 1.0 for full brightness; the
    /// constructor does not enforce that brightness range.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If an RGB channel is not an integer, `brightness` is not numeric, a keyword argument
    ///   is supplied, or the argument count is not exactly four.
    /// - `ValueError`: If an RGB channel is outside the inclusive range 0 to 255.
    #[make_new]
    #[stub(sig = "(self, r: int, g: int, b: int, brightness: float) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(4, 4).assert_nkw(0, 0);

        let r = reader.next_positional()?;
        let g = reader.next_positional()?;
        let b = reader.next_positional()?;
        let brightness = reader.next_positional()?;

        Ok(Self {
            base: ObjBase::new(ty),
            r,
            g,
            b,
            brightness,
        })
    }

    #[attr]
    #[stub(attrs = ["brightness: float", "r: int", "g: int", "b: int"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
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

    // more optimized than Eq according to Godbolt on armv7a-none-eabi
    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.brightness == rhs.brightness && lhs.r == rhs.r && lhs.g == rhs.g && lhs.b == rhs.b
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.try_as_obj::<Self>().unwrap())),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "LedMode.Manual(r={}, g={}, b={}, brightness={})",
            self.r, self.g, self.b, self.brightness,
        );
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
