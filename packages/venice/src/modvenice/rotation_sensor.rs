use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::{math::Direction, smart::rotation::RotationSensor};

use crate::{
    args::Args,
    devices::{self, PortNumber},
    fun::{fun1, fun2, fun3},
    modvenice::{
        motor::direction::DirectionObj,
        raise_device_error,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct RotationSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, RotationSensor>,
}

pub static ROTATION_SENSOR_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(RotationSensor))
    .set_make_new(make_new_from_fn!(rotation_sensor_make_new))
    .set_locals_dict(const_dict![
        qstr!(MIN_DATA_INTERVAL_MS) => Obj::from_int(5),
        qstr!(TICKS_PER_REVOLUTION) => Obj::from_int(36000),
        qstr!(angle) => Obj::from_static(&fun2!(rotation_sensor_angle, &RotationSensorObj, &RotationUnitObj)),
        qstr!(position) => Obj::from_static(&fun2!(rotation_sensor_position, &RotationSensorObj, &RotationUnitObj)),
        qstr!(set_position) => Obj::from_static(&fun3!(rotation_sensor_set_position, &RotationSensorObj, f32, &RotationUnitObj)),
        qstr!(velocity) => Obj::from_static(&fun1!(rotation_sensor_velocity, &RotationSensorObj)),
        qstr!(reset_position) => Obj::from_static(&fun1!(rotation_sensor_reset_position,&RotationSensorObj)),
        qstr!(set_direction) => Obj::from_static(&fun2!(rotation_sensor_set_direction,&RotationSensorObj, &DirectionObj)),
        qstr!(direction) => Obj::from_static(&fun1!(rotation_sensor_direction,&RotationSensorObj)),
        qstr!(status) => Obj::from_static(&fun1!(rotation_sensor_status,&RotationSensorObj)),
        qstr!(set_data_interval) => Obj::from_static(&fun3!(rotation_sensor_set_data_interval,&RotationSensorObj, f32, &TimeUnitObj)),
        qstr!(free) => Obj::from_static(&fun1!(rotation_sensor_free, &RotationSensorObj)),
    ]);

unsafe impl ObjTrait for RotationSensorObj {
    const OBJ_TYPE: &ObjType = ROTATION_SENSOR_OBJ_TYPE.as_obj_type();
}

fn rotation_sensor_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token().unwrap();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(1, 2).assert_nkw(0, 0);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let direction = reader
        .next_positional_or(&DirectionObj::FORWARD)
        .direction();

    let guard = devices::lock_port(port, |port| RotationSensor::new(port, direction));

    alloc_obj(RotationSensorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn rotation_sensor_angle(this: &RotationSensorObj, unit: &RotationUnitObj) -> Obj {
    let angle = this
        .guard
        .borrow_mut()
        .angle()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(unit.unit().in_angle(angle))
}

fn rotation_sensor_position(this: &RotationSensorObj, unit: &RotationUnitObj) -> Obj {
    let position = this
        .guard
        .borrow_mut()
        .position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(unit.unit().in_angle(position))
}

fn rotation_sensor_set_position(
    this: &RotationSensorObj,
    position: f32,
    unit: &RotationUnitObj,
) -> Obj {
    let angle = unit.unit().from_float(position);
    this.guard
        .borrow_mut()
        .set_position(angle)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn rotation_sensor_velocity(this: &RotationSensorObj) -> Obj {
    let velocity = this
        .guard
        .borrow_mut()
        .velocity()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(velocity as f32)
}

fn rotation_sensor_reset_position(this: &RotationSensorObj) -> Obj {
    this.guard
        .borrow_mut()
        .reset_position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn rotation_sensor_set_direction(this: &RotationSensorObj, direction: &DirectionObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_direction(direction.direction())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn rotation_sensor_direction(this: &RotationSensorObj) -> Obj {
    let dir = this.guard.borrow().direction();
    match dir {
        Direction::Forward => Obj::from_static(&DirectionObj::FORWARD),
        Direction::Reverse => Obj::from_static(&DirectionObj::REVERSE),
    }
}

fn rotation_sensor_status(this: &RotationSensorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(status as i32)
}

fn rotation_sensor_set_data_interval(
    this: &RotationSensorObj,
    interval: f32,
    unit: &TimeUnitObj,
) -> Obj {
    this.guard
        .borrow_mut()
        .set_data_interval(unit.unit().from_float(interval))
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn rotation_sensor_free(this: &RotationSensorObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
