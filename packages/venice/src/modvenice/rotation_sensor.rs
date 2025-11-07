use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    fun::{Fun1, Fun2, Fun3},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::{math::Direction, smart::rotation::RotationSensor};

use crate::{
    args::{ArgType, ArgValue, Args},
    devices::{PortNumber, try_lock_port},
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
    base: ObjBase,
    guard: RegistryGuard<'static, RotationSensor>,
}

pub static ROTATION_SENSOR_OBJ_TYPE: ObjFullType = ObjFullType::new(
    TypeFlags::empty(),
    qstr!(RotationSensor),
)
.set_slot_make_new(rotation_sensor_make_new)
.set_slot_locals_dict_from_static(&const_dict![
    qstr!(angle) => Obj::from_static(&Fun2::new(rotation_sensor_angle)),
    qstr!(position) => Obj::from_static(&Fun1::new(rotation_sensor_position)),
    qstr!(set_position) => Obj::from_static(&Fun3::new(rotation_sensor_set_position)),
    qstr!(velocity) => Obj::from_static(&Fun1::new(rotation_sensor_velocity)),
    qstr!(reset_position) => Obj::from_static(&Fun1::new(rotation_sensor_reset_position)),
    qstr!(set_direction) => Obj::from_static(&Fun2::new(rotation_sensor_set_direction)),
    qstr!(direction) => Obj::from_static(&Fun1::new(rotation_sensor_direction)),
    qstr!(status) => Obj::from_static(&Fun1::new(rotation_sensor_status)),
    qstr!(set_data_interval) => Obj::from_static(&Fun3::new(rotation_sensor_set_data_interval)),
]);

unsafe impl ObjTrait for RotationSensorObj {
    const OBJ_TYPE: &ObjType = ROTATION_SENSOR_OBJ_TYPE.as_obj_type();
}

extern "C" fn rotation_sensor_make_new(
    _: *const ObjType,
    n_pos: usize,
    n_kw: usize,
    ptr: *const Obj,
) -> Obj {
    let token = token().unwrap();
    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, ptr) }.reader(token);

    let port = PortNumber::from_i32(args.next_positional(ArgType::Int).as_int())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let direction = args
        .next_positional_or(
            ArgType::Obj(DirectionObj::OBJ_TYPE),
            ArgValue::Obj(Obj::from_static(&DirectionObj::FORWARD)),
        )
        .as_obj()
        .try_to_obj::<DirectionObj>()
        .unwrap()
        .direction();

    let guard = try_lock_port(port, |port| RotationSensor::new(port, direction))
        .unwrap_or_else(|_| raise_device_error(token, "port is already in use"));

    alloc_obj(RotationSensorObj {
        base: ObjBase::new(ROTATION_SENSOR_OBJ_TYPE.as_obj_type()),
        guard,
    })
}

extern "C" fn rotation_sensor_angle(self_in: Obj, unit: Obj) -> Obj {
    let token = token().unwrap();
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let unit = unit
        .try_to_obj::<RotationUnitObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <RotationUnit> for argument #1, found <{}>",
                    ArgType::of(&unit)
                ),
            )
        })
        .unit();
    let angle = sensor
        .guard
        .borrow_mut()
        .angle()
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::from_float(unit.in_angle(angle))
}

extern "C" fn rotation_sensor_position(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let position = sensor
        .guard
        .borrow_mut()
        .position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(position.as_radians() as f32)
}

extern "C" fn rotation_sensor_set_position(self_in: Obj, position: Obj, unit: Obj) -> Obj {
    let token = token().unwrap();
    let position_float = position.try_to_float().unwrap_or_else(|| {
        raise_type_error(
            token,
            format!(
                "expected <float> for argument #1, found <{}>",
                ArgType::of(&position)
            ),
        )
    });
    let unit_obj = unit.try_to_obj::<RotationUnitObj>().unwrap_or_else(|| {
        raise_type_error(
            token,
            format!(
                "expected <RotationUnit> for argument #2, found <{}>",
                ArgType::of(&unit)
            ),
        )
    });
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let angle = unit_obj.unit().from_float(position_float);
    sensor
        .guard
        .borrow_mut()
        .set_position(angle)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn rotation_sensor_velocity(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let velocity = sensor
        .guard
        .borrow_mut()
        .velocity()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(velocity as f32)
}

extern "C" fn rotation_sensor_reset_position(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    sensor
        .guard
        .borrow_mut()
        .reset_position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

extern "C" fn rotation_sensor_set_direction(self_in: Obj, direction: Obj) -> Obj {
    let token = token().unwrap();
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let dir = direction
        .try_to_obj::<DirectionObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <Direction> for argument #1, found <{}>",
                    ArgType::of(&direction)
                ),
            )
        })
        .direction();
    sensor
        .guard
        .borrow_mut()
        .set_direction(dir)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn rotation_sensor_direction(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let dir = sensor.guard.borrow().direction();
    match dir {
        Direction::Forward => Obj::from_static(&DirectionObj::FORWARD),
        Direction::Reverse => Obj::from_static(&DirectionObj::REVERSE),
    }
}

extern "C" fn rotation_sensor_status(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let status = sensor
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(status as i32)
}

extern "C" fn rotation_sensor_set_data_interval(self_in: Obj, interval: Obj, unit: Obj) -> Obj {
    let token = token().unwrap();
    let interval_float = interval.try_to_float().unwrap_or_else(|| {
        raise_type_error(
            token,
            format!(
                "expected <float> for argument #1, found <{}>",
                ArgType::of(&interval)
            ),
        )
    });
    let unit_obj = unit.try_to_obj::<TimeUnitObj>().unwrap_or_else(|| {
        raise_type_error(
            token,
            format!(
                "expected <TimeUnit> for argument #2, found <{}>",
                ArgType::of(&unit)
            ),
        )
    });
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    sensor
        .guard
        .borrow_mut()
        .set_data_interval(unit_obj.unit().from_float(interval_float))
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}
