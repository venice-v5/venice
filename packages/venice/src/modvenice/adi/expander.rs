use std::cell::Cell;

use argparse::{ArgParser, Args, ObjParser, ParseError, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::{adi::AdiPort, smart::expander::AdiExpander};

use crate::{
    devices::{self, AdiPortNumberParser},
    modvenice::{Exception, read_only_attr::read_only_attr},
};

/// A one-use reference to one ADI socket on an `AdiExpander`.
///
/// Users receive these objects through the expander's `a` through `h` attributes and pass them to ADI
/// device constructors. A port is consumed by the first device constructed from it and cannot be
/// constructed directly.
#[class(qstr!(AdiExpanderPort))]
pub struct AdiExpanderPortObj {
    base: ObjBase,
    port: Cell<Option<AdiPort>>,
}

impl From<AdiPort> for AdiExpanderPortObj {
    fn from(value: AdiPort) -> Self {
        Self {
            port: Cell::new(Some(value)),
            base: Self::OBJ_TYPE.into(),
        }
    }
}

/// Provides eight additional ADI ports from one V5 Smart Port.
///
/// ADI expanders are identical to the built-in three-wire ports on the Brain, with the exception that
/// ports on an expander will not work properly if the Brain can't verify that the expander is
/// connected and valid.
///
/// The read-only attributes `a`, `b`, `c`, `d`, `e`, `f`, `g`, and `h` each contain the matching
/// `AdiExpanderPort`. Each attribute may be consumed once by an ADI device constructor. Operations on
/// devices made from these ports raise `DeviceError` if the expander is disconnected or the Smart
/// Port contains another device type.
#[class(qstr!(AdiExpander))]
pub struct AdiExpanderObj {
    base: ObjBase,
    adi_a: Obj,
    adi_b: Obj,
    adi_c: Obj,
    adi_d: Obj,
    adi_e: Obj,
    adi_f: Obj,
    adi_g: Obj,
    adi_h: Obj,
}

#[class_methods]
impl AdiExpanderPortObj {}

#[class_methods]
impl AdiExpanderObj {
    /// Creates a new ADI expander on Smart Port `port`.
    ///
    /// `port` is an integer from 1 through 21. Construction reserves that Smart Port immediately; the
    /// expander itself does not check whether the hardware is connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// expander = AdiExpander(1)
    /// analog = AdiAnalogIn(expander.a)
    /// print(analog.get_voltage())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is outside 1 through 21 or is already occupied.
    #[make_new]
    #[stub(sig = "(self, port: int) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port_number = reader.next_positional()?;
        let expander = devices::lock_port(port_number, AdiExpander::new)
            .take()
            .unwrap();

        Ok(Self {
            base: ty.into(),
            adi_a: AdiExpanderPortObj::from(expander.adi_a).into(),
            adi_b: AdiExpanderPortObj::from(expander.adi_b).into(),
            adi_c: AdiExpanderPortObj::from(expander.adi_c).into(),
            adi_d: AdiExpanderPortObj::from(expander.adi_d).into(),
            adi_e: AdiExpanderPortObj::from(expander.adi_e).into(),
            adi_f: AdiExpanderPortObj::from(expander.adi_f).into(),
            adi_g: AdiExpanderPortObj::from(expander.adi_g).into(),
            adi_h: AdiExpanderPortObj::from(expander.adi_h).into(),
        })
    }

    #[attr]
    #[stub(attrs = [
        "a: AdiExpanderPort",
        "b: AdiExpanderPort",
        "c: AdiExpanderPort",
        "d: AdiExpanderPort",
        "e: AdiExpanderPort",
        "f: AdiExpanderPort",
        "g: AdiExpanderPort",
        "h: AdiExpanderPort",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };

        result.return_value(match attr.as_str() {
            "a" => self.adi_a,
            "b" => self.adi_b,
            "c" => self.adi_c,
            "d" => self.adi_d,
            "e" => self.adi_e,
            "f" => self.adi_f,
            "g" => self.adi_g,
            "h" => self.adi_h,
            _ => return,
        });
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AdiPortParser;

impl<'a> ArgParser<'a> for AdiPortParser {
    type Output = AdiPort;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        match AdiPortNumberParser.parse(obj) {
            Ok(number) => {
                return devices::try_lock_adi_port(number).map_err(|_| ParseError::ValueError {
                    mk_msg: Box::new(move |arg| {
                        error_msg!("{arg}: adi port '{number}' is occupied by another device")
                    }),
                });
            }
            Err(e) => match e {
                ParseError::ValueError { mk_msg } => return Err(ParseError::ValueError { mk_msg }),
                ParseError::TypeError { .. } => {}
            },
        };

        let parser = ObjParser::<AdiExpanderPortObj>::default();
        parser.parse(obj).and_then(|o| {
            o.port.take().ok_or(ParseError::ValueError {
                mk_msg: Box::new(|arg| {
                    error_msg!("{arg}: adi expander port is occupied by another device")
                }),
            })
        })
    }
}
