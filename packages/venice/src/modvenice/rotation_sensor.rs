use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::{math::Direction, smart::rotation::RotationSensor};

use crate::{
    devices::{self},
    modvenice::{
        Exception,
        motor::direction::DirectionObj,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    registry::SmartGuard,
};

#[class(qstr!(RotationSensor))]
#[repr(C)]
pub struct RotationSensorObj {
    base: ObjBase,
    guard: SmartGuard<RotationSensor>,
}

#[class_methods]
impl RotationSensorObj {
    #[constant]
    const MIN_DATA_INTERVAL_MS: i32 = RotationSensor::MIN_DATA_INTERVAL.as_millis() as i32;

    #[constant]
    const TICKS_PER_REVOLUTION: i32 = RotationSensor::TICKS_PER_REVOLUTION as i32;

    #[make_new]
    fn new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 2).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let direction = reader
            .next_positional_or(DirectionObj::FORWARD)?
            .direction();

        let guard = devices::lock_port(port, |port| RotationSensor::new(port, direction));

        Ok(RotationSensorObj {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let angle = self.guard.borrow_mut().angle()?;
        Ok(unit.unit().angle_to_float(angle))
    }

    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let position = self.guard.borrow_mut().position()?;
        Ok(unit.unit().angle_to_float(position))
    }

    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position);
        self.guard.borrow_mut().set_position(angle)?;
        Ok(())
    }

    #[method]
    fn get_velocity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow_mut().velocity()? as f32)
    }

    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        self.guard.borrow_mut().reset_position()?;
        Ok(())
    }

    #[method]
    fn set_direction(&self, direction: &DirectionObj) -> Result<(), Exception> {
        self.guard
            .borrow_mut()
            .set_direction(direction.direction())?;
        Ok(())
    }

    // Venice's convention for deciding between using getters/setters and attributes is that
    // attributes should never perform SDK calls. If loading or storing some `x` requires calling
    // into the SDK, then that functionality should be moved into getters and setters `get_x` and
    // `set_x`.
    //
    // Loading rotation sensor direction does not require an SDK call, but storing it does
    // (`set_direction`). It's possible to define `direction` as an attribute and make it
    // read-only, but clash with the separate setter API and be misleading for users. That's why,
    // despite not requiring an SDK call, loading direction is still a getter method instead of an
    // attribute.
    #[method]
    fn get_direction(&self) -> Obj {
        let dir = self.guard.borrow().direction();
        Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        })
    }

    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        self.guard
            .borrow_mut()
            .set_data_interval(unit.unit().float_to_dur(interval))?;
        Ok(())
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
