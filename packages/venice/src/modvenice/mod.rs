mod adi;
mod ai_vision;
mod battery;
mod color;
mod competition;
mod controller;
mod distance_sensor;
mod gps;
mod imu;
mod math;
mod motor;
mod optical;
mod rotation_sensor;
mod serial;
mod units;
mod vasyncio;
mod vision;

use argparse::{KeywordError, PositionalError, error_msg};
use micropython_rs::{
    const_map,
    except::{EXCEPTION_TYPE, ExceptionType, Message},
    init::InitToken,
    map::Dict,
    module::Module,
    obj::{Obj, ObjTrait},
};
use vex_sdk::V5_MAX_DEVICE_PORTS;
use vex_sdk_jumptable::{V5_DeviceT, V5_DeviceType, vexDeviceGetByIndex, vexDeviceGetStatus};
use vexide_devices::smart::{PortError, SmartDeviceType};

use crate::modvenice::{
    adi::{
        accelerometer::{AdiAccelerometerObj, SensitivityObj},
        analog::AdiAnalogInObj,
        digital::{AdiDigitalInObj, AdiDigitalOutObj},
        encoder::AdiEncoderObj,
        expander::{AdiExpanderObj, AdiExpanderPortObj},
        gyroscope::{AdiGyroscopeFuture, AdiGyroscopeObj},
        light_sensor::AdiLightSensorObj,
        line_tracker::AdiLineTrackerObj,
        motor::AdiMotorObj,
        pwm::AdiPwmOutObj,
        servo::AdiServoObj,
    },
    ai_vision::{
        AiVisionSensorObj, color::AiVisionColorObj, color_code::AiVisionColorCodeObj,
        detection_mode::AiVisionDetectionModeObj, flags::AiVisionFlagsObj,
    },
    battery::BATTERY_DICT,
    color::ColorObj,
    competition::{Competition, CompetitionRuntime},
    controller::{
        ControllerObj,
        id::ControllerIdObj,
        state::{ButtonStateObj, ControllerStateObj, JoystickStateObj},
    },
    distance_sensor::{DistanceSensorObj, distance_object::DistanceObjectObj},
    gps::GpsSensorObj,
    imu::{InertialOrientationObj, InertialSensorObj},
    math::{EulerZYX, Point2, Quaternion, Vec3},
    motor::{
        MotorObj, brake::BrakeModeObj, direction::DirectionObj, gearset::GearsetObj,
        motor_type::MotorTypeObj,
    },
    optical::{
        OpticalSensorObj,
        gesture::{GestureDirectionObj, GestureObj},
        rgb::{OpticalRawObj, OpticalRgbObj},
    },
    rotation_sensor::RotationSensorObj,
    serial::{SerialPortObj, SerialPortOpenFutureObj},
    units::{rotation::RotationUnitObj, time::TimeUnitObj},
    vasyncio::VASYNCIO_DICT,
    vision::{
        VisionSensorObj, code::VisionCodeObj, led_mode::LedModeObj, mode::VisionModeObj,
        object::VisionObjectObj, signature::VisionSignatureObj, source::DetectionSourceObj,
        white_balance::WhiteBalanceObj,
    },
};

static DEVICE_ERROR_TYPE: ExceptionType = ExceptionType::new(qstr!(DeviceError), EXCEPTION_TYPE);

pub struct Exception(pub micropython_rs::except::Exception);

impl Exception {
    pub fn new(ty: &'static ExceptionType, msg: impl Into<Message>) -> Self {
        Self(micropython_rs::except::Exception {
            ty,
            msg: msg.into(),
        })
    }

    pub fn raise(&self, token: InitToken) -> ! {
        self.0.raise(token);
    }
}

impl From<micropython_rs::except::Exception> for Exception {
    fn from(value: micropython_rs::except::Exception) -> Self {
        Self(value)
    }
}

impl From<Exception> for micropython_rs::except::Exception {
    fn from(value: Exception) -> Self {
        value.0
    }
}

impl From<PositionalError<'_>> for Exception {
    fn from(value: PositionalError<'_>) -> Self {
        Self(value.into())
    }
}

impl From<KeywordError<'_>> for Exception {
    fn from(value: KeywordError<'_>) -> Self {
        Self(value.into())
    }
}

impl From<PortError> for Exception {
    fn from(value: PortError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

pub fn device_error(msg: impl Into<Message>) -> Exception {
    Exception::new(&DEVICE_ERROR_TYPE, msg)
}

fn smart_port_index(n: u8) -> u32 {
    (n - 1) as u32
}

unsafe fn device_handle(index: u32) -> V5_DeviceT {
    unsafe { vexDeviceGetByIndex(index) }
}

/// Verify that the device type is currently plugged into this port.
///
/// This function provides the internal implementations of [`SmartDevice::validate_port`],
/// [`SmartPort::validate_type`], and [`AdiPort::validate_expander`].
fn validate_port(number: u8, device_type: SmartDeviceType) -> Result<(), PortError> {
    let mut device_types: [V5_DeviceType; V5_MAX_DEVICE_PORTS] = unsafe { core::mem::zeroed() };
    unsafe {
        vexDeviceGetStatus(device_types.as_mut_ptr());
    }

    let connected_type: Option<SmartDeviceType> = match device_types[(number - 1) as usize] {
        V5_DeviceType::kDeviceTypeNoSensor => None,
        raw_type => Some(raw_type.into()),
    };

    if let Some(connected_type) = connected_type {
        // The connected device must match the requested type.
        if connected_type != device_type {
            return Err(PortError::IncorrectDevice {
                expected: device_type,
                actual: connected_type,
                port: number,
            });
        }
    } else {
        // No device is plugged into the port.
        return Err(PortError::Disconnected { port: number });
    }

    Ok(())
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static mut venice_globals: Dict = Dict::new(const_map![
    qstr!(__name__) => Obj::from_qstr(qstr!(venice)),

    // motor
    qstr!(Motor) => Obj::from_static(MotorObj::OBJ_TYPE),
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
    qstr!(AiVisionColorObject) => Obj::from_static(ai_vision::object::Color::OBJ_TYPE),
    qstr!(AiVisionCodeObject) => Obj::from_static(ai_vision::object::Code::OBJ_TYPE),
    qstr!(AiVisionAprilTagObject) => Obj::from_static(ai_vision::object::AprilTag::OBJ_TYPE),
    qstr!(AiVisionModelObject) => Obj::from_static(ai_vision::object::Model::OBJ_TYPE),
    // competition
    qstr!(Competition) => Obj::from_static(Competition::OBJ_TYPE),
    qstr!(CompetitionRuntime) => Obj::from_static(CompetitionRuntime::OBJ_TYPE),
    // imu
    qstr!(InertialSensor) => Obj::from_static(InertialSensorObj::OBJ_TYPE),
    qstr!(InertialOrientation) => Obj::from_static(InertialOrientationObj::OBJ_TYPE),
    // optical
    qstr!(OpticalSensor) => Obj::from_static(OpticalSensorObj::OBJ_TYPE),
    qstr!(OpticalRgb) => Obj::from_static(OpticalRgbObj::OBJ_TYPE),
    qstr!(OpticalRaw) => Obj::from_static(OpticalRawObj::OBJ_TYPE),
    qstr!(Gesture) => Obj::from_static(GestureObj::OBJ_TYPE),
    qstr!(GestureDirection) => Obj::from_static(GestureDirectionObj::OBJ_TYPE),
    // serial
    qstr!(SerialPort) => Obj::from_static(SerialPortObj::OBJ_TYPE),
    qstr!(SerialPortOpenFuture) => Obj::from_static(SerialPortOpenFutureObj::OBJ_TYPE),
    // vision
    qstr!(VisionSensor) => Obj::from_static(VisionSensorObj::OBJ_TYPE),
    qstr!(VisionCode) => Obj::from_static(VisionCodeObj::OBJ_TYPE),
    qstr!(LedMode) => Obj::from_static(LedModeObj::OBJ_TYPE),
    qstr!(VisionMode) => Obj::from_static(VisionModeObj::OBJ_TYPE),
    qstr!(VisionObject) => Obj::from_static(VisionObjectObj::OBJ_TYPE),
    qstr!(VisionSignature) => Obj::from_static(VisionSignatureObj::OBJ_TYPE),
    qstr!(DetectionSource) => Obj::from_static(DetectionSourceObj::OBJ_TYPE),
    qstr!(WhiteBalance) => Obj::from_static(WhiteBalanceObj::OBJ_TYPE),
    // other devices
    qstr!(RotationSensor) => Obj::from_static(RotationSensorObj::OBJ_TYPE),
    qstr!(GpsSensor) => Obj::from_static(GpsSensorObj::OBJ_TYPE),

    // adi
    qstr!(AdiMotor) => Obj::from_static(AdiMotorObj::OBJ_TYPE),
    qstr!(AdiGyroscope) => Obj::from_static(AdiGyroscopeObj::OBJ_TYPE),
    qstr!(AdiGyroscopeFuture) => Obj::from_static(AdiGyroscopeFuture::OBJ_TYPE),
    qstr!(AdiDigitalIn) => Obj::from_static(AdiDigitalInObj::OBJ_TYPE),
    qstr!(AdiDigitalOut) => Obj::from_static(AdiDigitalOutObj::OBJ_TYPE),
    qstr!(AdiExpander) => Obj::from_static(AdiExpanderObj::OBJ_TYPE),
    qstr!(AdiExpanderPort) => Obj::from_static(AdiExpanderPortObj::OBJ_TYPE),
    qstr!(AdiEncoder) => Obj::from_static(AdiEncoderObj::OBJ_TYPE),
    qstr!(AdiPwmOut) => Obj::from_static(AdiPwmOutObj::OBJ_TYPE),
    qstr!(AdiAnalogIn) => Obj::from_static(AdiAnalogInObj::OBJ_TYPE),
    qstr!(AdiAccelerometer) => Obj::from_static(AdiAccelerometerObj::OBJ_TYPE),
    qstr!(AdiAccelerometerSensitivity) => Obj::from_static(SensitivityObj::OBJ_TYPE),
    qstr!(AdiLightSensor) => Obj::from_static(AdiLightSensorObj::OBJ_TYPE),
    qstr!(AdiLineTracker) => Obj::from_static(AdiLineTrackerObj::OBJ_TYPE),
    qstr!(AdiServo) => Obj::from_static(AdiServoObj::OBJ_TYPE),

    // submodules
    qstr!(vasyncio) => Obj::from_static(&Module::new(VASYNCIO_DICT)),
    qstr!(battery) => Obj::from_static(&Module::new(BATTERY_DICT)),

    // math
    qstr!(Vec3) => Obj::from_static(Vec3::OBJ_TYPE),
    qstr!(Quaternion) => Obj::from_static(Quaternion::OBJ_TYPE),
    qstr!(EulerZYX) => Obj::from_static(EulerZYX::OBJ_TYPE),
    qstr!(Point2) => Obj::from_static(Point2::OBJ_TYPE),
    // color
    qstr!(Color) => Obj::from_static(ColorObj::OBJ_TYPE),

    // units
    qstr!(RotationUnit) => Obj::from_static(RotationUnitObj::OBJ_TYPE),
    qstr!(TimeUnit) => Obj::from_static(TimeUnitObj::OBJ_TYPE)
]);
