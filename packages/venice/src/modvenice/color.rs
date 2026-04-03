use std::cell::Cell;

use argparse::{ArgParser, Args, IntParser, ParseError, error_msg, type_name};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{type_error, value_error},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjType},
    qstr::Qstr,
};
use vexide_devices::color::Color;

use crate::modvenice::Exception;

#[class(qstr!(Color))]
#[repr(C)]
pub struct ColorObj {
    base: ObjBase,
    r: Cell<u8>,
    g: Cell<u8>,
    b: Cell<u8>,
}

#[class_methods]
impl ColorObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 3).assert_nkw(0, 0);

        let r = reader.next_positional_or(0)?;
        let g = reader.next_positional_or(0)?;
        let b = reader.next_positional_or(0)?;

        Ok(Self {
            base: ty.into(),
            r: r.into(),
            g: g.into(),
            b: b.into(),
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let channel = match attr.as_str() {
            "r" => &self.r,
            "g" => &self.g,
            "b" => &self.b,
            _ => return,
        };

        match op {
            AttrOp::Load { result } => result.return_value(channel.get() as i32),
            AttrOp::Store { src, result } => {
                let parser = IntParser::<u8>::default();
                match parser.parse(&src) {
                    Ok(v) => {
                        channel.set(v);
                        result.success();
                    }
                    Err(e) => match e {
                        ParseError::TypeError { expected } => type_error(error_msg!(
                            "expected '{expected}', found '{}'",
                            type_name(&src)
                        )),
                        ParseError::ValueError { mk_msg } => value_error(mk_msg("h")),
                    }
                    .raise(token()),
                }
            }
            AttrOp::Delete { result } => {
                channel.set(0);
                result.sucess();
            }
        }
    }

    pub fn color(&self) -> Color {
        Color::new(self.r.get(), self.g.get(), self.b.get())
    }

    #[method]
    fn as_int(&self) -> i32 {
        self.color().into_raw() as i32
    }
}
