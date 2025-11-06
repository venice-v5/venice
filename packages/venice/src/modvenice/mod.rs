mod controller;
mod motor;
mod rotation_sensor;
mod units;

use micropython_rs::{
    const_dict,
    except::{new_exception_type, raise_msg},
    init::InitToken,
    map::Dict,
    obj::{Obj, ObjFullType, ObjTrait},
};

use self::{
    controller::ControllerObj,
    motor::{MotorObj, brake::BrakeModeObj, direction::DirectionObj, gearset::GearsetObj},
    rotation_sensor::RotationSensorObj,
    units::rotation::RotationUnitObj,
};
use crate::qstrgen::qstr;

static DEVICE_ERROR_OBJ_TYPE: ObjFullType = new_exception_type(qstr!(DeviceError));

fn raise_device_error(token: InitToken, msg: impl AsRef<str>) -> ! {
    raise_msg(token, DEVICE_ERROR_OBJ_TYPE.as_obj_type(), msg)
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static venice_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(__name__)),

    qstr!(Motor) => Obj::from_static(MotorObj::OBJ_TYPE),
    qstr!(Gearset) => Obj::from_static(GearsetObj::OBJ_TYPE),
    qstr!(Direction) => Obj::from_static(DirectionObj::OBJ_TYPE),
    qstr!(BrakeMode) => Obj::from_static(BrakeModeObj::OBJ_TYPE),

    qstr!(Controller) => Obj::from_static(ControllerObj::OBJ_TYPE),

    qstr!(RotationUnit) => Obj::from_static(RotationUnitObj::OBJ_TYPE),
    qstr!(RotationSensor) => Obj::from_static(RotationSensorObj::OBJ_TYPE),
];
