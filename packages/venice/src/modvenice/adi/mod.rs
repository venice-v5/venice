use vex_sdk_jumptable::{V5_DeviceT, vexDeviceAdiPortConfigSet, vexDeviceGetByIndex};
use vexide_devices::{
    adi::{AdiDeviceType, AdiPort},
    smart::{PortError, SmartDeviceType},
};

use crate::modvenice::validate_port;

pub mod accelerometer;
pub mod addrled;
pub mod analog;
pub mod digital;
pub mod encoder;
pub mod expander;
pub mod gyroscope;
pub mod light_sensor;
pub mod line_tracker;
pub mod motor;
pub mod potentiometer;
pub mod pwm;
pub mod range_finder;
pub mod servo;

const INTERNAL_ADI_PORT_NUMBER: u8 = 22;

fn validate_expander(expander_number: Option<u8>) -> Result<(), PortError> {
    validate_port(
        expander_number.unwrap_or(INTERNAL_ADI_PORT_NUMBER),
        SmartDeviceType::Adi,
    )
}

pub fn expander_index(expander_number: Option<u8>) -> u32 {
    u32::from((expander_number.unwrap_or(INTERNAL_ADI_PORT_NUMBER)) - 1)
}

fn adi_port_index(number: u8) -> u32 {
    (number - 1) as u32
}

fn adi_port_name(port: u8) -> char {
    match port {
        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        5 => 'E',
        6 => 'F',
        7 => 'G',
        8 => 'H',
        _ => '?',
    }
}

fn device_handle(port: &AdiPort) -> V5_DeviceT {
    unsafe { vexDeviceGetByIndex(expander_index(port.expander_number())) }
}

fn configure_port(port: &AdiPort, config: AdiDeviceType) {
    unsafe {
        vexDeviceAdiPortConfigSet(
            device_handle(port),
            adi_port_index(port.number()),
            config.into(),
        );
    }
}
