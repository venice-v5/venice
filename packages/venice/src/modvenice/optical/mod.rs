pub mod gesture;
pub mod rgb;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::optical::OpticalSensor;

use crate::{
    devices,
    modvenice::{
        Exception,
        optical::{
            gesture::GestureObj,
            rgb::{OpticalRawObj, OpticalRgbObj},
        },
        units::time::TimeUnitObj,
    },
    registry::RegistryGuard,
};

#[class(qstr!(OpticalSensor))]
#[repr(C)]
pub struct OpticalSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, OpticalSensor>,
}

#[class_methods]
impl OpticalSensorObj {
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

        Ok(OpticalSensorObj {
            base: ObjBase::new(ty),
            guard: devices::lock_port(port, |p| OpticalSensor::new(p)),
        })
    }

    #[method]
    fn get_hue(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().hue()? as f32)
    }

    #[method]
    fn get_saturation(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().saturation()? as f32)
    }

    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().brightness()? as f32)
    }

    #[method]
    fn get_proximity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().proximity()? as f32)
    }

    #[method]
    fn get_color(&self) -> Result<OpticalRgbObj, Exception> {
        Ok(OpticalRgbObj::new(self.guard.borrow().color()?))
    }

    #[method]
    fn get_raw_color(&self) -> Result<OpticalRawObj, Exception> {
        Ok(OpticalRawObj::new(self.guard.borrow().raw_color()?))
    }

    #[method]
    fn get_last_gesture(&self) -> Result<Option<GestureObj>, Exception> {
        Ok(self.guard.borrow().last_gesture()?.map(GestureObj::new))
    }

    #[method]
    fn get_led_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().led_brightness()? as f32)
    }

    #[method]
    fn set_led_brightness(&self, brightness: f32) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_led_brightness(brightness as f64)?)
    }

    #[method]
    fn get_integration_time(&self, unit: &TimeUnitObj) -> Result<f32, Exception> {
        Ok(unit
            .unit()
            .dur_to_float(self.guard.borrow().integration_time()?))
    }

    #[method]
    fn set_integration_time(&self, time: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_integration_time(unit.unit().float_to_dur(time))?)
    }

    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32) // should be OK to cast, the bits themselves stay the same
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
