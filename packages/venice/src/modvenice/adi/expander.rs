use std::cell::RefCell;

use argparse::{ArgParser, Args, ObjParser, ParseError, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::{adi::AdiPort, smart::expander::AdiExpander};

use crate::{
    devices::{self, AdiPortNumber, AdiPortNumberParser},
    lifecycle::{AdiPortIdentity, AdiPortPlanError, validate_adi_port_plan},
    modvenice::{Exception, read_only_attr::read_only_attr},
};

/// A one-use reference to one ADI socket on an `AdiExpander`.
///
/// Users receive these objects through the expander's `a` through `h` attributes and pass them to ADI
/// device constructors. A port is consumed by the first device successfully constructed from it;
/// failed argument parsing or validation leaves it available. Ports cannot be constructed directly.
#[class(qstr!(AdiExpanderPort))]
pub struct AdiExpanderPortObj {
    base: ObjBase,
    // Keep the identity after consumption so parsing a used port stays non-destructive and can
    // report occupancy during the transaction's validation phase.
    port: RefCell<Option<AdiPort>>,
    number: u8,
    expander_number: Option<u8>,
}

impl From<AdiPort> for AdiExpanderPortObj {
    fn from(value: AdiPort) -> Self {
        Self {
            number: value.number(),
            expander_number: value.expander_number(),
            port: RefCell::new(Some(value)),
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
/// `AdiExpanderPort`. Each attribute may be consumed once by a successful ADI device constructor;
/// failed construction leaves it available. Operations on devices made from these ports raise
/// `DeviceError` if the expander is disconnected or the Smart Port contains another device type.
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
    #[stub(sig = "(self, port: int, /) -> None")]
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

#[derive(Clone, Copy)]
enum AdiPortSource<'a> {
    Onboard(AdiPortNumber),
    Expander(&'a AdiExpanderPortObj),
}

/// A parsed ADI port that has not yet consumed its underlying resource.
///
/// Constructor invariants are enforced in three phases: parse every argument into specs and plain
/// values, validate relationships and availability, then consume the specs without any intervening
/// fallible Python operation. This is required because a MicroPython non-local raise can bypass
/// automatic cleanup, so rollback during cleanup isn't sufficient here.
#[derive(Clone, Copy)]
pub struct AdiPortSpec<'a> {
    source: AdiPortSource<'a>,
    number: u8,
    expander_number: Option<u8>,
}

impl AdiPortSpec<'_> {
    pub fn number(self) -> u8 {
        self.number
    }

    pub fn expander_number(self) -> Option<u8> {
        self.expander_number
    }

    fn identity(self) -> AdiPortIdentity {
        AdiPortIdentity::new(self.number, self.expander_number)
    }

    fn is_available(self) -> bool {
        match self.source {
            AdiPortSource::Onboard(number) => devices::adi_port_is_available(number),
            AdiPortSource::Expander(port) => port.port.borrow().is_some(),
        }
    }

    fn take(self) -> AdiPort {
        match self.source {
            AdiPortSource::Onboard(number) => devices::try_lock_adi_port(number)
                .expect("validated onboard ADI port became unavailable during commit"),
            AdiPortSource::Expander(port) => port
                .port
                .borrow_mut()
                .take()
                .expect("validated expander ADI port became unavailable during commit"),
        }
    }

    pub fn commit(self) -> Result<AdiPort, Exception> {
        validate_adi_port_plan([(self.identity(), self.is_available())])
            .map_err(|error| plan_error(error, &[self]))?;
        Ok(self.take())
    }
}

fn plan_error(error: AdiPortPlanError, specs: &[AdiPortSpec<'_>]) -> Exception {
    match error {
        AdiPortPlanError::Unavailable(index) => {
            micropython_rs::except::value_error(match specs[index].source {
                AdiPortSource::Onboard(number) => {
                    error_msg!("adi port '{number}' is occupied by another device")
                }
                AdiPortSource::Expander(_) => {
                    error_msg!("adi expander port is occupied by another device")
                }
            })
            .into()
        }
        AdiPortPlanError::Duplicate => {
            micropython_rs::except::value_error(c"the same ADI port cannot be used twice").into()
        }
    }
}

pub fn commit_adi_port_pair(
    first: AdiPortSpec<'_>,
    second: AdiPortSpec<'_>,
) -> Result<(AdiPort, AdiPort), Exception> {
    // Checking both resources before either take makes this a commit phase. Pair constructors must
    // perform all argument parsing and relationship validation before calling this function.
    validate_adi_port_plan([
        (first.identity(), first.is_available()),
        (second.identity(), second.is_available()),
    ])
    .map_err(|error| plan_error(error, &[first, second]))?;
    Ok((first.take(), second.take()))
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AdiPortParser;

impl<'a> ArgParser<'a> for AdiPortParser {
    type Output = AdiPortSpec<'a>;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        match AdiPortNumberParser.parse(obj) {
            Ok(number) => {
                return Ok(AdiPortSpec {
                    source: AdiPortSource::Onboard(number),
                    number: number.number(),
                    expander_number: None,
                });
            }
            Err(ParseError::ValueError { mk_msg }) => {
                return Err(ParseError::ValueError { mk_msg });
            }
            Err(ParseError::TypeError { .. }) => {}
        };

        let port = ObjParser::<AdiExpanderPortObj>::default().parse(obj)?;
        Ok(AdiPortSpec {
            source: AdiPortSource::Expander(port),
            number: port.number,
            expander_number: port.expander_number,
        })
    }
}
