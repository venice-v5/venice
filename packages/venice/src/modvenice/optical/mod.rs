pub mod gesture;
pub mod rgb;

use argparse::{Args, Exception};
use micropython_rs::{
    class, class_methods,
    init::token,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::optical::OpticalSensor;

use crate::{
    devices,
    modvenice::{
        optical::{
            gesture::GestureObj,
            rgb::{OpticalRawObj, OpticalRgbObj},
        },
        raise_port_error,
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
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;

        Ok(OpticalSensorObj {
            base: ObjBase::new(ty),
            guard: devices::lock_port(port, |p| OpticalSensor::new(p)),
        })
    }

    #[method]
    fn get_hue(&self) -> f32 {
        self.guard
            .borrow()
            .hue()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_saturation(&self) -> f32 {
        self.guard
            .borrow()
            .saturation()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_brightness(&self) -> f32 {
        self.guard
            .borrow()
            .brightness()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_proximity(&self) -> f32 {
        self.guard
            .borrow()
            .proximity()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_color(&self) -> OpticalRgbObj {
        OpticalRgbObj::new(
            self.guard
                .borrow()
                .color()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn get_raw_color(&self) -> OpticalRawObj {
        OpticalRawObj::new(
            self.guard
                .borrow()
                .raw_color()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn get_last_gesture(&self) -> Option<GestureObj> {
        self.guard
            .borrow()
            .last_gesture()
            .unwrap_or_else(|e| raise_port_error!(e))
            .map(GestureObj::new)
    }

    #[method]
    fn get_led_brightness(&self) -> f32 {
        self.guard
            .borrow()
            .led_brightness()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn set_led_brightness(&self, brightness: f32) {
        self.guard
            .borrow_mut()
            .set_led_brightness(brightness as f64)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_integration_time(&self, unit: &TimeUnitObj) -> f32 {
        unit.unit().dur_to_float(
            self.guard
                .borrow()
                .integration_time()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn set_integration_time(&self, time: f32, unit: &TimeUnitObj) {
        self.guard
            .borrow_mut()
            .set_integration_time(unit.unit().float_to_dur(time))
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_status(&self) -> i32 {
        self.guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e)) as i32 // should be OK to cast, the bits themselves stay the same
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
