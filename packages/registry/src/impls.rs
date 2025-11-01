use vexide_devices::smart::{
    SmartPort, ai_vision::AiVisionSensor, distance::DistanceSensor, electromagnet::Electromagnet,
    expander::AdiExpander, gps::GpsSensor, imu::InertialSensor, link::RadioLink, motor::Motor,
    optical::OpticalSensor, rotation::RotationSensor, serial::SerialPort, vision::VisionSensor,
};

use crate::Device;

macro_rules! impl_device {
    ($($device:ty),*) => {
        $(
            impl Device for $device {
                fn take_port(self) -> vexide_devices::smart::SmartPort
                where
                    Self: Sized,
                {
                    SmartPort::from(self)
                }
            }
        )*
    };
}

impl_device!(
    Motor,
    RotationSensor,
    DistanceSensor,
    Electromagnet,
    InertialSensor,
    RadioLink,
    GpsSensor,
    AdiExpander,
    AiVisionSensor,
    VisionSensor,
    SerialPort,
    OpticalSensor
);
