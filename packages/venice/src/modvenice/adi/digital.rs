use std::cell::RefCell;

use argparse::{Args, PositionalError};
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::digital::{AdiDigitalIn, AdiDigitalOut, LogicLevel};

use crate::{devices, modvenice::Exception};

#[class(qstr!(AdiDigitalIn))]
#[repr(C)]
pub struct AdiDigitalInObj {
    base: ObjBase,
    r#in: AdiDigitalIn,
}

#[class(qstr!(AdiDigitalOut))]
#[repr(C)]
pub struct AdiDigitalOutObj {
    base: ObjBase,
    out: RefCell<AdiDigitalOut>,
}

fn level_to_bool(level: LogicLevel) -> bool {
    match level {
        LogicLevel::High => true,
        LogicLevel::Low => false,
    }
}

fn bool_to_level(b: bool) -> LogicLevel {
    match b {
        true => LogicLevel::High,
        false => LogicLevel::Low,
    }
}

#[class_methods]
impl AdiDigitalInObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        Ok(Self {
            base: ty.into(),
            r#in: AdiDigitalIn::new(devices::lock_adi_port(port)),
        })
    }

    #[method]
    fn get_value(&self) -> Result<bool, Exception> {
        Ok(level_to_bool(self.r#in.level()?))
    }

    #[method]
    fn is_high(&self) -> Result<bool, Exception> {
        Ok(self.r#in.is_high()?)
    }

    #[method]
    fn is_low(&self) -> Result<bool, Exception> {
        Ok(self.r#in.is_low()?)
    }
}

#[class_methods]
impl AdiDigitalOutObj {
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
        let initial_level = match reader.next_positional::<bool>() {
            Ok(v) => Some(v),
            Err(e) => match e {
                PositionalError::ArgumentsExhausted => None,
                _ => Err(e)?,
            },
        };

        let port = devices::lock_adi_port(port_number);
        let out = match initial_level {
            Some(level) => AdiDigitalOut::with_initial_level(port, bool_to_level(level)),
            None => AdiDigitalOut::new(port),
        };

        Ok(Self {
            base: ty.into(),
            out: RefCell::new(out),
        })
    }

    #[method]
    fn get_value(&self) -> Result<bool, Exception> {
        Ok(level_to_bool(self.out.borrow().level()?))
    }

    #[method]
    fn set_value(&self, value: bool) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_level(bool_to_level(value))?)
    }

    #[method]
    fn set_high(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_high()?)
    }

    #[method]
    fn set_low(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().set_low()?)
    }

    #[method]
    fn toggle(&self) -> Result<(), Exception> {
        Ok(self.out.borrow_mut().toggle()?)
    }
}
