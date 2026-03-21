use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::gps::GpsSensor;

use crate::{
    devices,
    modvenice::{
        Exception,
        math::{EulerAngles, Point2, Quaternion, Vec3},
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    registry::RegistryGuard,
};

#[class(qstr!(GpsSensor))]
#[repr(C)]
pub struct GpsSensorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, GpsSensor>,
}

#[class_methods]
impl GpsSensorObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();

        let port_number = reader.next_positional()?;
        let offset = reader.next_positional::<&Point2>()?;
        let initial_position = reader.next_positional::<&Point2>()?;
        let initial_heading = reader.next_positional::<f32>()?;
        let initial_heading_unit = reader.next_positional::<&RotationUnitObj>()?;

        let initial_heading_angle = initial_heading_unit.unit().float_to_angle(initial_heading);

        Ok(Self {
            guard: devices::lock_port(port_number, |port| {
                GpsSensor::new(
                    port,
                    offset.as_vexide_point2(),
                    initial_position.as_vexide_point2(),
                    initial_heading_angle.as_radians(),
                )
            }),
            base: ty.into(),
        })
    }

    #[method]
    fn get_offset(&self) -> Result<Point2, Exception> {
        Ok(self.guard.borrow().offset()?.into())
    }

    #[method]
    fn set_offset(&self, offset: &Point2) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_offset(offset.as_vexide_point2())?)
    }

    #[method]
    fn get_position(&self) -> Result<Point2, Exception> {
        Ok(self.guard.borrow().position()?.into())
    }

    #[method]
    fn get_heading(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().heading()?))
    }

    #[method]
    fn set_heading(&self, heading: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_heading(unit.unit().float_to_angle(heading))?)
    }

    #[method]
    fn reset_heading(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_heading()?)
    }

    #[method]
    fn get_rotation(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().rotation()?))
    }

    #[method]
    fn set_rotation(&self, rotation: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_rotation(unit.unit().float_to_angle(rotation))?)
    }

    #[method]
    fn reset_rotation(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_rotation()?)
    }

    #[method]
    fn get_euler(&self, unit: &RotationUnitObj) -> Result<EulerAngles, Exception> {
        Ok(EulerAngles::new(self.guard.borrow().euler()?, unit.unit()))
    }

    #[method]
    fn get_quaternion(&self) -> Result<Quaternion, Exception> {
        Ok(Quaternion::new(self.guard.borrow().quaternion()?))
    }

    #[method]
    fn get_acceleration(&self) -> Result<Vec3, Exception> {
        Ok(Vec3::new(self.guard.borrow().acceleration()?))
    }

    #[method]
    fn get_gyro_rate(&self) -> Result<Vec3, Exception> {
        Ok(Vec3::new(self.guard.borrow().gyro_rate()?))
    }

    #[method]
    fn get_error(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().error()? as f32)
    }

    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        // cast from u32 to i32, should be OK since the amount of bits is preserved and no data is
        // lost
        Ok(self.guard.borrow().status()? as i32)
    }

    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_data_interval(unit.unit().float_to_dur(interval))?)
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
