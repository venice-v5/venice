use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::electromagnet::Electromagnet;

use crate::{
    devices,
    modvenice::{Exception, units::time::TimeUnitObj},
    registry::SmartGuard,
};

#[class(qstr!(Motor))]
pub struct ElectromagnetObj {
    base: ObjBase,
    guard: SmartGuard<Electromagnet>,
}

#[class_methods]
impl ElectromagnetObj {
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
        Ok(Self {
            base: ty.into(),
            guard: devices::lock_port(port_number, Electromagnet::new),
        })
    }

    #[method(ty = var_between(min = 3, max = 3))]
    fn set_power(args: &[Obj]) -> Result<(), Exception> {
        let mut reader = Args::new(3, 0, args).reader();
        let this = reader.next_positional::<&Self>()?;

        let power = reader.next_positional::<f32>()?;
        let duration = reader.next_positional()?;
        let time_unit = reader.next_positional::<&TimeUnitObj>()?;

        Ok(this
            .guard
            .borrow_mut()
            .set_power(power as f64, time_unit.unit().float_to_dur(duration))?)
    }

    #[method]
    fn get_power(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().power()? as f32)
    }

    #[method]
    fn get_current(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current()? as f32)
    }

    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
