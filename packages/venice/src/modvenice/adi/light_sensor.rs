use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::light_sensor::AdiLightSensor;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

#[class(qstr!(AdiLightSensor))]
pub struct AdiLightSensorObj {
    base: ObjBase,
    sensor: AdiLightSensor,
}

#[class_methods]
impl AdiLightSensorObj {
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
            sensor: AdiLightSensor::new(port),
        })
    }

    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.sensor.brightness()? as f32)
    }

    #[method]
    fn get_raw_brightness(&self) -> Result<i32, Exception> {
        Ok(self.sensor.raw_brightness()? as i32)
    }
}
