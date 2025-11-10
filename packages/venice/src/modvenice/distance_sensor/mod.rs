pub mod distance_object;

use micropython_rs::{
    const_dict, except::raise_value_error, init::token, make_new_from_fn, obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags}
};
use vexide_devices::smart::distance::DistanceSensor;

use crate::{
    args::Args, devices::{PortNumber, try_lock_port}, fun::fun1_from_fn, modvenice::{distance_sensor::distance_object::DistanceObjectObj, raise_device_error}, obj::alloc_obj, qstrgen::qstr, registry::RegistryGuard
};

#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, DistanceSensor>,
}

pub static DISTANCE_SENSOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(DistanceSensor))
        .set_make_new(make_new_from_fn!(distance_sensor_make_new))
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(object) => Obj::from_static(&fun1_from_fn!(fn distance_sensor_object(&DistanceSensorObj))),
            qstr!(status) => Obj::from_static(&fun1_from_fn!(fn distance_sensor_status(&DistanceSensorObj))),
        ]);

unsafe impl ObjTrait for DistanceSensorObj {
    const OBJ_TYPE: &ObjType = DISTANCE_SENSOR_OBJ_TYPE.as_obj_type();
}

fn distance_sensor_make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token().unwrap();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(1, 1).assert_nkw(0, 0);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let guard = try_lock_port(port, |port| DistanceSensor::new(port))
        .unwrap_or_else(|_| raise_device_error(token, "port is already in use"));

    alloc_obj(DistanceSensorObj {
        base: ObjBase::new(DISTANCE_SENSOR_OBJ_TYPE.as_obj_type()),
        guard,
    })
}

fn distance_sensor_status(this: &DistanceSensorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(status as i32)
}

fn distance_sensor_object(this: &DistanceSensorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .object()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    match status {
        Some(state) => alloc_obj(DistanceObjectObj::new(state)),
        None => Obj::NONE,
    }
}
