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
    modvenice::Exception,
};

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
    #[make_new]
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
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            todo!("need standardize error messages for ro attr structs")
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
