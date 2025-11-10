pub mod april_tag_family;

use micropython_rs::{
    const_dict,
    except::raise_value_error,
    fun::Fun1,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::ai_vision::AiVisionSensor;

use crate::{
    args::Args,
    devices::{PortNumber, try_lock_port},
    modvenice::{distance_sensor::distance_object::DistanceObjectObj, raise_device_error},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct AiVisionSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, AiVisionSensor>,
}

pub static AI_VISION_SENSOR_OBJ_TYPE: ObjFullType = unsafe {
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionSensor))
        .set_slot_make_new(ai_vision_sensor_make_new)
        // .set_slot_locals_dict_from_static(&const_dict![
        //     qstr!(object) => Obj::from_static(&Fun1::new(distance_sensor_object)),
        //     qstr!(status) => Obj::from_static(&Fun1::new(distance_sensor_status)),
        // ])
};

unsafe impl ObjTrait for AiVisionSensorObj {
    const OBJ_TYPE: &ObjType = AI_VISION_SENSOR_OBJ_TYPE.as_obj_type();
}

unsafe extern "C" fn ai_vision_sensor_make_new(
    _: *const ObjType,
    n_pos: usize,
    n_kw: usize,
    ptr: *const Obj,
) -> Obj {
    let token = token().unwrap();
    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, ptr) }.reader(token);

    let port = PortNumber::from_i32(args.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let guard = try_lock_port(port, |port| AiVisionSensor::new(port))
        .unwrap_or_else(|_| raise_device_error(token, "port is already in use"));

    alloc_obj(AiVisionSensorObj {
        base: ObjBase::new(AI_VISION_SENSOR_OBJ_TYPE.as_obj_type()),
        guard,
    })
}
