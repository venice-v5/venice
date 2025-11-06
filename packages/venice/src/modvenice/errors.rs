use std::ffi::c_void;

use micropython_rs::{
    except::{mp_obj_exception_attr, mp_obj_exception_make_new, mp_obj_exception_print, mp_type_BaseException, raise_msg, raise_value_error},
    init::token,
    obj::{ObjFullType, ObjType, TypeFlags},
};
use vexide_devices::smart::{PortError, SmartDeviceType};

use crate::qstrgen::qstr;

static DEVICE_ERROR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(DeviceError))
        .set_slot_parent(&raw const mp_type_BaseException as *const c_void)
        .set_slot_attr(mp_obj_exception_attr)
        .set_slot_print(mp_obj_exception_print)
        .set_slot_make_new(mp_obj_exception_make_new);

pub fn raise_device(e: PortError) -> ! {
    let error = match e {
        PortError::Disconnected { port } => format!("Port {} is disconnected", port),
        PortError::IncorrectDevice {
            expected,
            actual,
            port,
        } => format!(
            "Incorrect device type: expected {} but got {} connected to port {}",
            device_type(&expected),
            device_type(&actual),
            port
        ),
    };
    raise_msg(token().unwrap(), DEVICE_ERROR_OBJ_TYPE.as_obj_type(), error)
}

fn device_type(dev: &SmartDeviceType) -> String {
    match dev {
        SmartDeviceType::Motor => "motor".to_string(),
        SmartDeviceType::Rotation => "rotation sensor".to_string(),
        SmartDeviceType::Imu => "inertial sensor".to_string(),
        SmartDeviceType::Distance => "distance sensor".to_string(),
        SmartDeviceType::Vision => "vision sensor".to_string(),
        SmartDeviceType::AiVision => "AI Vision sensor".to_string(),
        SmartDeviceType::Electromagnet => "electromagnet".to_string(),
        SmartDeviceType::LightTower => "light tower".to_string(),
        SmartDeviceType::Arm => "arm".to_string(),
        SmartDeviceType::Optical => "optical".to_string(),
        SmartDeviceType::Gps => "gps".to_string(),
        SmartDeviceType::Radio => "radio".to_string(),
        SmartDeviceType::Adi => "ADI expander".to_string(),
        SmartDeviceType::GenericSerial => "generic serial device".to_string(),
        SmartDeviceType::Unknown(..) => "unknown".to_string(),
    }
}
