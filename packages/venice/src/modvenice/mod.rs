mod ai_vision;
mod competition;
mod controller;
mod distance_sensor;
mod motor;
mod rotation_sensor;
pub mod units;

use std::ffi::CStr;

use micropython_rs::{
    const_map,
    except::{mp_type_Exception, new_exception_type, raise_msg},
    init::InitToken,
    map::Dict,
    obj::{Obj, ObjFullType, ObjTrait},
};

use self::{
    controller::{
        ControllerObj,
        state::{ButtonStateObj, ControllerStateObj, JoystickStateObj},
    },
    motor::{MotorObj, brake::BrakeModeObj, direction::DirectionObj, gearset::GearsetObj},
    rotation_sensor::RotationSensorObj,
    units::{rotation::RotationUnitObj, time::TimeUnitObj},
};
use crate::{
    modvenice::{
        ai_vision::{
            AiVisionSensorObj,
            ai_vision_color::AiVisionColorObj,
            ai_vision_color_code::AiVisionColorCodeObj,
            ai_vision_detection_mode::AiVisionDetectionModeObj,
            ai_vision_flags::AiVisionFlagsObj,
            ai_vision_object::{
                AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE, AI_VISION_CODE_OBJECT_OBJ_TYPE,
                AI_VISION_COLOR_OBJECT_OBJ_TYPE, AI_VISION_MODEL_OBJECT_OBJ_TYPE,
            },
        },
        competition::{COMPETITION_OBJ_TYPE, COMPETITION_RUNTIME_OBJ_TYPE},
        controller::id::ControllerIdObj,
        distance_sensor::{DistanceSensorObj, distance_object::DistanceObjectObj},
        motor::{MOTOR_EXP_OBJ_TYPE, MOTOR_V5_OBJ_TYPE, motor_type::MotorTypeObj},
    },
    qstrgen::qstr,
};

static DEVICE_ERROR_OBJ_TYPE: ObjFullType =
    new_exception_type(qstr!(DeviceError), &mp_type_Exception);

pub fn raise_device_error(token: InitToken, msg: impl AsRef<CStr>) -> ! {
    raise_msg(token, DEVICE_ERROR_OBJ_TYPE.as_obj_type(), msg)
}

macro_rules! raise_port_error {
    ($e:expr) => {
        $crate::modvenice::raise_device_error(
            ::micropython_rs::init::token(),
            $crate::error_msg::error_msg!("{}", $e),
        )
    };
}

pub(crate) use raise_port_error;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static mut venice_globals: Dict = Dict::new(const_map![
    qstr!(__name__) => Obj::from_qstr(qstr!(__name__)),

    // motor
    qstr!(AbstractMotor) => Obj::from_static(MotorObj::OBJ_TYPE),
    qstr!(MotorV5) => Obj::from_static(&MOTOR_V5_OBJ_TYPE),
    qstr!(MotorExp) => Obj::from_static(&MOTOR_EXP_OBJ_TYPE),
    qstr!(Gearset) => Obj::from_static(GearsetObj::OBJ_TYPE),
    qstr!(BrakeMode) => Obj::from_static(BrakeModeObj::OBJ_TYPE),
    qstr!(Direction) => Obj::from_static(DirectionObj::OBJ_TYPE),
    qstr!(MotorType) => Obj::from_static(MotorTypeObj::OBJ_TYPE),
    // controller
    qstr!(Controller) => Obj::from_static(ControllerObj::OBJ_TYPE),
    qstr!(ControllerId) => Obj::from_static(ControllerIdObj::OBJ_TYPE),
    qstr!(ControllerState) => Obj::from_static(ControllerStateObj::OBJ_TYPE),
    qstr!(ButtonState) => Obj::from_static(ButtonStateObj::OBJ_TYPE),
    qstr!(JoystickState) => Obj::from_static(JoystickStateObj::OBJ_TYPE),
    // distance
    qstr!(DistanceObject) => Obj::from_static(DistanceObjectObj::OBJ_TYPE),
    qstr!(DistanceSensor) => Obj::from_static(DistanceSensorObj::OBJ_TYPE),
    // ai vision sensor
    qstr!(AiVisionColor) => Obj::from_static(AiVisionColorObj::OBJ_TYPE),
    qstr!(AiVisionColorCode) => Obj::from_static(AiVisionColorCodeObj::OBJ_TYPE),
    qstr!(AiVisionDetectionMode) => Obj::from_static(AiVisionDetectionModeObj::OBJ_TYPE),
    qstr!(AiVisionFlags) => Obj::from_static(AiVisionFlagsObj::OBJ_TYPE),
    qstr!(AiVisionSensor) => Obj::from_static(AiVisionSensorObj::OBJ_TYPE),
    qstr!(AiVisionColorObject) => Obj::from_static(&AI_VISION_COLOR_OBJECT_OBJ_TYPE),
    qstr!(AiVisionCodeObject) => Obj::from_static(&AI_VISION_CODE_OBJECT_OBJ_TYPE),
    qstr!(AiVisionAprilTagObject) => Obj::from_static(&AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE),
    qstr!(AiVisionModelObject) => Obj::from_static(&AI_VISION_MODEL_OBJECT_OBJ_TYPE),
    // competition
    qstr!(Competition) => Obj::from_static(&COMPETITION_OBJ_TYPE),
    qstr!(CompetitionRuntime) => Obj::from_static(&COMPETITION_RUNTIME_OBJ_TYPE),
    // other devices
    qstr!(RotationSensor) => Obj::from_static(RotationSensorObj::OBJ_TYPE),

    // units
    qstr!(RotationUnit) => Obj::from_static(RotationUnitObj::OBJ_TYPE),
    qstr!(TimeUnit) => Obj::from_static(TimeUnitObj::OBJ_TYPE)
]);
