use std::cell::RefCell;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::motor::AdiMotor;

use crate::{devices, modvenice::Exception};

#[class(qstr!(AdiMotor))]
#[repr(C)]
pub struct AdiMotorObj {
    base: ObjBase,
    motor: RefCell<AdiMotor>,
}

#[class_methods]
impl AdiMotorObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        let port = reader.next_positional()?;
        // TODO: should this be made optional? If so, what should be its default value?
        let slew = reader.next_positional()?;

        Ok(Self {
            base: ty.into(),
            motor: RefCell::new(AdiMotor::new(devices::lock_adi_port(port), slew)),
        })
    }

    #[method]
    fn set_output(&self, value: f32) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().set_output(value as f64)?)
    }

    #[method]
    fn set_raw_output(&self, pwm: i8) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().set_raw_output(pwm)?)
    }

    #[method]
    fn get_output(&self) -> Result<f32, Exception> {
        Ok(self.motor.borrow().output()? as f32)
    }

    #[method]
    fn get_raw_output(&self) -> Result<i32, Exception> {
        Ok(self.motor.borrow().raw_output()? as i32)
    }

    #[method]
    fn stop(&self) -> Result<(), Exception> {
        Ok(self.motor.borrow_mut().stop()?)
    }
}
