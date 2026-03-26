use vexide_devices::smart::{PortError, SmartDeviceType};

use crate::modvenice::validate_port;

pub mod analog;
pub mod digital;
pub mod encoder;
pub mod expander;
pub mod gyroscope;
pub mod motor;
pub mod pwm;

const INTERNAL_ADI_PORT_NUMBER: u8 = 22;

fn validate_expander(expander_number: Option<u8>) -> Result<(), PortError> {
    validate_port(
        expander_number.unwrap_or(INTERNAL_ADI_PORT_NUMBER),
        SmartDeviceType::Adi,
    )
}

fn adi_port_index(number: u8) -> u32 {
    (number - 1) as u32
}
