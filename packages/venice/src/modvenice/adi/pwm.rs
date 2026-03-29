use std::cell::RefCell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::pwm::AdiPwmOut;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

#[class(qstr!(AdiPwmOut))]
pub struct AdiPwmOutObj {
    base: ObjBase,
    pwm: RefCell<AdiPwmOut>,
}

#[class_methods]
impl AdiPwmOutObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        Ok(Self {
            base: ty.into(),
            pwm: RefCell::new(AdiPwmOut::new(port)),
        })
    }

    #[method]
    fn set_output(&self, value: u8) -> Result<(), Exception> {
        Ok(self.pwm.borrow_mut().set_output(value)?)
    }
}
