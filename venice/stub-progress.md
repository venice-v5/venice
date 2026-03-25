[DONE] qstr!(Motor) => Obj::from_static(MotorObj::OBJ_TYPE),
[DONE] qstr!(Gearset) => Obj::from_static(GearsetObj::OBJ_TYPE),
[DONE] qstr!(BrakeMode) => Obj::from_static(BrakeModeObj::OBJ_TYPE),
[DONE] qstr!(Direction) => Obj::from_static(DirectionObj::OBJ_TYPE),
[DONE] qstr!(MotorType) => Obj::from_static(MotorTypeObj::OBJ_TYPE),
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
qstr!(AiVisionColorObject) => Obj::from_static(ai_vision_object::Color::OBJ_TYPE),
qstr!(AiVisionCodeObject) => Obj::from_static(ai_vision_object::Code::OBJ_TYPE),
qstr!(AiVisionAprilTagObject) => Obj::from_static(ai_vision_object::AprilTag::OBJ_TYPE),
qstr!(AiVisionModelObject) => Obj::from_static(ai_vision_object::Model::OBJ_TYPE),
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

// async
qstr!(EventLoop) => Obj::from_static(EventLoop::OBJ_TYPE),
qstr!(Sleep) => Obj::from_static(Sleep::OBJ_TYPE),
qstr!(get_running_loop) => Obj::from_static(&Fun0::new(vasyncio_get_running_loop)),
qstr!(run) => Obj::from_static(&Fun1::new(vasyncio_run)),
qstr!(spawn) => Obj::from_static(&Fun1::new(vasyncio_spawn)),

// math
qstr!(Vec3) => Obj::from_static(Vec3::OBJ_TYPE),
qstr!(Quaternion) => Obj::from_static(Quaternion::OBJ_TYPE),
qstr!(EulerAngles) => Obj::from_static(EulerAngles::OBJ_TYPE),

// units
qstr!(RotationUnit) => Obj::from_static(RotationUnitObj::OBJ_TYPE),
qstr!(TimeUnit) => Obj::from_static(TimeUnitObj::OBJ_TYPE)
