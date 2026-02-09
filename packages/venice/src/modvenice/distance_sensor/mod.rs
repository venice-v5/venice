pub mod distance_object;

use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::distance::DistanceSensor;

use crate::{
    args::Args,
    devices::{self, PortNumber},
    fun::fun1,
    modvenice::{distance_sensor::distance_object::DistanceObjectObj, raise_device_error},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, DistanceSensor>,
}

pub static DISTANCE_SENSOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(DistanceSensor))
        .set_make_new(make_new_from_fn!(distance_sensor_make_new))
        .set_locals_dict(const_dict![
            qstr!(object) => Obj::from_static(&fun1!(distance_sensor_object, &DistanceSensorObj)),
            qstr!(status) => Obj::from_static(&fun1!(distance_sensor_status, &DistanceSensorObj)),
            qstr!(free) => Obj::from_static(&fun1!(distance_sensor_free, &DistanceSensorObj)),
        ]);

unsafe impl ObjTrait for DistanceSensorObj {
    const OBJ_TYPE: &ObjType = DISTANCE_SENSOR_OBJ_TYPE.as_obj_type();
}

fn distance_sensor_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(1, 1).assert_nkw(0, 0);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let guard = devices::lock_port(port, DistanceSensor::new);

    alloc_obj(DistanceSensorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn distance_sensor_status(this: &DistanceSensorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token(), format!("{e}")));
    Obj::from_int(status as i32)
}

fn distance_sensor_object(this: &DistanceSensorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .object()
        .unwrap_or_else(|e| raise_device_error(token(), format!("{e}")));
    match status {
        Some(state) => alloc_obj(DistanceObjectObj::new(state)),
        None => Obj::NONE,
    }
}

fn distance_sensor_free(this: &DistanceSensorObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
