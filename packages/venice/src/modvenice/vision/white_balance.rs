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
use vexide_devices::smart::vision::WhiteBalance;

use crate::modvenice::{Exception, read_only_attr::read_only_attr};

/// Vision Sensor white balance mode.
///
/// Represents a white balance configuration for the Vision Sensor's camera. `WhiteBalance` is an
/// abstract base and cannot be instantiated directly. Construct its associated-only variants as
/// `WhiteBalance.Auto()`, `WhiteBalance.StartupAuto()`, or `WhiteBalance.Manual(...)`; the variants are
/// not package-root imports.
#[class(qstr!(WhiteBalance))]
#[repr(C)]
pub struct WhiteBalanceObj {
    base: ObjBase,
}

/// Automatic Mode.
///
/// The sensor will automatically adjust the camera's white balance, using the brightest part of the
/// image as a white point. This associated-only class is constructed as `WhiteBalance.Auto()` and is
/// not package-root importable. Construction always returns the same singleton, which prints as
/// `WhiteBalance.Auto()`.
#[class(qstr!(Auto))]
#[repr(C)]
pub struct Auto {
    base: ObjBase,
}

/// "Startup" Automatic Mode.
///
/// The sensor will automatically adjust the camera's white balance, but will only perform this
/// adjustment once on power-on. This associated-only class is constructed as
/// `WhiteBalance.StartupAuto()` and is not package-root importable. Construction always returns the
/// same singleton, which prints as `WhiteBalance.StartupAuto()`.
#[class(qstr!(StartupAuto))]
#[repr(C)]
pub struct StartupAuto {
    base: ObjBase,
}

/// Manual Mode.
///
/// This mode allows for manual control over white balance using an RGB color. This associated-only class is
/// constructed as `WhiteBalance.Manual(r, g, b)` and is not package-root importable. Its read-only
/// integer attributes `r`, `g`, and `b` are the red, green, and blue white-point channels from 0 to
/// 255. Values compare by these three channels, return `False` when compared with another type,
/// and print as `WhiteBalance.Manual(r=..., g=..., b=...)`.
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
    /// Rejects direct construction of the abstract `WhiteBalance` base class.
    ///
    /// Use one of the associated variant classes instead.
    ///
    /// # Raises
    ///
    /// - `TypeError`: Always.
    #[make_new]
    #[stub(sig = "(self, /) -> None")]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        type_error(
            c"WhiteBalance is an abstract base class; use WhiteBalance.Auto(), WhiteBalance.StartupAuto(), or WhiteBalance.Manual(...)",
        )
        .raise(token());
    }

    /// The associated Automatic Mode class, constructed as `WhiteBalance.Auto()`.
    #[constant(qstr!(Auto))]
    const AUTO: &ObjType = Auto::OBJ_TYPE;
    /// The associated "Startup" Automatic Mode class, constructed as `WhiteBalance.StartupAuto()`.
    #[constant(qstr!(StartupAuto))]
    const STARTUP_AUTO: &ObjType = StartupAuto::OBJ_TYPE;
    /// The associated Manual Mode class, constructed as `WhiteBalance.Manual(r, g, b)`.
    #[constant(qstr!(Manual))]
    const MANUAL: &ObjType = Manual::OBJ_TYPE;
}

#[class_methods]
impl Auto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    /// Returns the singleton Automatic Mode.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If any positional or keyword arguments are supplied.
    #[make_new]
    #[stub(sig = "(self, /) -> None")]
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("WhiteBalance.Auto()");
    }
}

#[class_methods]
impl StartupAuto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    /// Returns the singleton "Startup" Automatic Mode.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If any positional or keyword arguments are supplied.
    #[make_new]
    #[stub(sig = "(self, /) -> None")]
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("WhiteBalance.StartupAuto()")
    }
}

#[class_methods]
impl Manual {
    /// Creates a Manual Mode with RGB channels `r`, `g`, and `b`.
    ///
    /// Each channel is a positional-only integer from 0 to 255.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a channel is not an integer, a keyword argument is supplied, or the argument
    ///   count is not exactly three.
    /// - `ValueError`: If a channel is outside the inclusive range 0 to 255.
    #[make_new]
    #[stub(sig = "(self, r: int, g: int, b: int, /) -> None")]
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
    #[stub(attrs = ["r: int", "g: int", "b: int"])]
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

    // more optimized than Eq according to Godbolt on armv7a-none-eabi
    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.r == rhs.r && lhs.g == rhs.g && lhs.b == rhs.b
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(
                rhs.try_as_obj::<Self>()
                    .is_some_and(|rhs| Self::eq(lhs, rhs)),
            ),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "WhiteBalance.Manual(r={}, g={}, b={})",
            self.r, self.g, self.b
        );
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
