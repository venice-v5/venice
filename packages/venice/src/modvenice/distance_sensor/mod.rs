pub mod distance_object;

use micropython_rs::{const_dict, except::raise_value_error, fun::Fun1, init::token, obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags}};
use vexide_devices::smart::distance::DistanceSensor;

use crate::{args::Args, devices::{PortNumber, try_lock_port}, modvenice::{distance_sensor::distance_object::DistanceObjectObj, raise_device_error}, obj::alloc_obj, qstrgen::qstr, registry::RegistryGuard};

#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, DistanceSensor>,
}

pub static DISTANCE_SENSOR_OBJ_TYPE: ObjFullType = unsafe {
    ObjFullType::new(TypeFlags::empty(), qstr!(DistanceSensor))
        .set_slot_make_new(distance_sensor_make_new)
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(object) => Obj::from_static(&Fun1::new(distance_sensor_object)),
            qstr!(status) => Obj::from_static(&Fun1::new(distance_sensor_status)),
        ])
};

unsafe impl ObjTrait for DistanceSensorObj {
    const OBJ_TYPE: &ObjType = DISTANCE_SENSOR_OBJ_TYPE.as_obj_type();
}

unsafe extern "C" fn distance_sensor_make_new(
    _: *const ObjType,
    n_pos: usize,
    n_kw: usize,
    ptr: *const Obj,
) -> Obj {
    let token = token().unwrap();
    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, ptr) }.reader(token);

    let port = PortNumber::from_i32(args.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let guard = try_lock_port(port, |port| DistanceSensor::new(port))
        .unwrap_or_else(|_| raise_device_error(token, "port is already in use"));

    alloc_obj(DistanceSensorObj {
        base: ObjBase::new(DISTANCE_SENSOR_OBJ_TYPE.as_obj_type()),
        guard,
    })
}

extern "C" fn distance_sensor_status(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<DistanceSensorObj>().unwrap();
    let status = sensor
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(status as i32)
}

extern "C" fn distance_sensor_object(self_in: Obj) -> Obj {
    let sensor = self_in.try_to_obj::<DistanceSensorObj>().unwrap();
    let status = sensor
        .guard
        .borrow()
        .object()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    match status {
        Some(state) => alloc_obj(DistanceObjectObj::new(state)),
        None => Obj::NONE
    }
}
