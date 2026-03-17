use argparse::Args;
use micropython_rs::{
    class, class_methods,
    init::token,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::{math::Direction, smart::rotation::RotationSensor};

use crate::{
    devices::{self},
    modvenice::{
        Exception,
        motor::direction::DirectionObj,
        raise_port_error,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    registry::RegistryGuard,
};

#[class(qstr!(RotationSensor))]
#[repr(C)]
pub struct RotationSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, RotationSensor>,
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
        let token = token();
        let mut reader = Args::new(n_pos, n_kw, args).reader(token);
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
    fn get_angle(&self, unit: &RotationUnitObj) -> f32 {
        let angle = self
            .guard
            .borrow_mut()
            .angle()
            .unwrap_or_else(|e| raise_port_error!(e));
        unit.unit().angle_to_float(angle)
    }

    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Obj {
        let position = self
            .guard
            .borrow_mut()
            .position()
            .unwrap_or_else(|e| raise_port_error!(e));
        Obj::from_float(unit.unit().angle_to_float(position))
    }

    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) {
        let angle = unit.unit().float_to_angle(position);
        self.guard
            .borrow_mut()
            .set_position(angle)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_velocity(&self) -> f32 {
        let velocity = self
            .guard
            .borrow_mut()
            .velocity()
            .unwrap_or_else(|e| raise_port_error!(e));
        velocity as f32
    }

    #[method]
    fn reset_position(&self) {
        self.guard
            .borrow_mut()
            .reset_position()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_direction(&self, direction: &DirectionObj) {
        self.guard
            .borrow_mut()
            .set_direction(direction.direction())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_direction(&self) -> Obj {
        let dir = self.guard.borrow().direction();
        Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        })
    }

    #[method]
    fn get_status(&self) -> i32 {
        let status = self
            .guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e));
        status as i32
    }

    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) {
        self.guard
            .borrow_mut()
            .set_data_interval(unit.unit().float_to_dur(interval))
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
