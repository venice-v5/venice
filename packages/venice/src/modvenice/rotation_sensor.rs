use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    fun::{Fun1, Fun2},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::{math::Angle, smart::rotation::RotationSensor};

use crate::{
    args::{ArgType, ArgValue, Args},
    devices::{PortNumber, try_lock_port},
    modvenice::{motor::direction::DirectionObj, raise_device_error},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct RotationSensorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, RotationSensor>,
}

pub static ROTATION_SENSOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(RotationSensor))
        .set_slot_make_new(rotation_sensor_make_new)
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(angle) => Obj::from_static(&Fun1::new(rotation_sensor_angle)),
            qstr!(position) => Obj::from_static(&Fun1::new(rotation_sensor_position)),
            qstr!(set_position) => Obj::from_static(&Fun2::new(rotation_sensor_set_position)),
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

extern "C" fn rotation_sensor_angle(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    let angle = sensor
        .guard
        .borrow_mut()
        .angle()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(angle.as_radians() as f32)
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

extern "C" fn rotation_sensor_set_position(self_in: Obj, position: Obj) -> Obj {
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
    let sensor = self_in.try_to_obj::<RotationSensorObj>().unwrap();
    sensor
        .guard
        .borrow_mut()
        .set_position(Angle::from_radians(position_float as f64))
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}
