use std::{
    fmt::Write,
    sync::{LazyLock, Mutex, MutexGuard},
};

use argparse::{ArgParser, DefaultParser, IntParser, ParseError, StrParser, error_msg};
use vexide_devices::{
    adi::AdiPort, controller::ControllerId, display::Display, peripherals::Peripherals,
    smart::SmartPort,
};

use crate::registry::{
    AdiRegistry, ControllerGuard, ControllerRegistry, DeviceOccupiedError, PortDevice, Registry,
    RegistryGuard, SmartRegistry,
};

pub struct Devices {
    pub primary_controller: ControllerRegistry,
    pub partner_controller: ControllerRegistry,

    pub port_1: SmartRegistry,
    pub port_2: SmartRegistry,
    pub port_3: SmartRegistry,
    pub port_4: SmartRegistry,
    pub port_5: SmartRegistry,
    pub port_6: SmartRegistry,
    pub port_7: SmartRegistry,
    pub port_8: SmartRegistry,
    pub port_9: SmartRegistry,
    pub port_10: SmartRegistry,
    pub port_11: SmartRegistry,
    pub port_12: SmartRegistry,
    pub port_13: SmartRegistry,
    pub port_14: SmartRegistry,
    pub port_15: SmartRegistry,
    pub port_16: SmartRegistry,
    pub port_17: SmartRegistry,
    pub port_18: SmartRegistry,
    pub port_19: SmartRegistry,
    pub port_20: SmartRegistry,
    pub port_21: SmartRegistry,

    pub adi_a: AdiRegistry,
    pub adi_b: AdiRegistry,
    pub adi_c: AdiRegistry,
    pub adi_d: AdiRegistry,
    pub adi_e: AdiRegistry,
    pub adi_f: AdiRegistry,
    pub adi_g: AdiRegistry,
    pub adi_h: AdiRegistry,

    pub display: Mutex<Display>,
}

impl Devices {
    fn new() -> Option<Self> {
        Peripherals::take().map(|peris| Self {
            primary_controller: ControllerRegistry::new(peris.primary_controller),
            partner_controller: ControllerRegistry::new(peris.partner_controller),

            port_1: Registry::new(peris.port_1),
            port_2: Registry::new(peris.port_2),
            port_3: Registry::new(peris.port_3),
            port_4: Registry::new(peris.port_4),
            port_5: Registry::new(peris.port_5),
            port_6: Registry::new(peris.port_6),
            port_7: Registry::new(peris.port_7),
            port_8: Registry::new(peris.port_8),
            port_9: Registry::new(peris.port_9),
            port_10: Registry::new(peris.port_10),
            port_11: Registry::new(peris.port_11),
            port_12: Registry::new(peris.port_12),
            port_13: Registry::new(peris.port_13),
            port_14: Registry::new(peris.port_14),
            port_15: Registry::new(peris.port_15),
            port_16: Registry::new(peris.port_16),
            port_17: Registry::new(peris.port_17),
            port_18: Registry::new(peris.port_18),
            port_19: Registry::new(peris.port_19),
            port_20: Registry::new(peris.port_20),
            port_21: Registry::new(peris.port_21),

            adi_a: AdiRegistry::new(peris.adi_a),
            adi_b: AdiRegistry::new(peris.adi_b),
            adi_c: AdiRegistry::new(peris.adi_c),
            adi_d: AdiRegistry::new(peris.adi_d),
            adi_e: AdiRegistry::new(peris.adi_e),
            adi_f: AdiRegistry::new(peris.adi_f),
            adi_g: AdiRegistry::new(peris.adi_g),
            adi_h: AdiRegistry::new(peris.adi_h),

            display: Mutex::new(peris.display),
        })
    }

    fn registry_by_port(&self, port: PortNumber) -> &SmartRegistry {
        match port.number() {
            1 => &self.port_1,
            2 => &self.port_2,
            3 => &self.port_3,
            4 => &self.port_4,
            5 => &self.port_5,
            6 => &self.port_6,
            7 => &self.port_7,
            8 => &self.port_8,
            9 => &self.port_9,
            10 => &self.port_10,
            11 => &self.port_11,
            12 => &self.port_12,
            13 => &self.port_13,
            14 => &self.port_14,
            15 => &self.port_15,
            16 => &self.port_16,
            17 => &self.port_17,
            18 => &self.port_18,
            19 => &self.port_19,
            20 => &self.port_20,
            21 => &self.port_21,
            _ => unreachable!(),
        }
    }

    fn adi_registry_by_port(&self, port: AdiPortNumber) -> &AdiRegistry {
        match port {
            AdiPortNumber::A => &self.adi_a,
            AdiPortNumber::B => &self.adi_b,
            AdiPortNumber::C => &self.adi_c,
            AdiPortNumber::D => &self.adi_d,
            AdiPortNumber::E => &self.adi_e,
            AdiPortNumber::F => &self.adi_f,
            AdiPortNumber::G => &self.adi_g,
            AdiPortNumber::H => &self.adi_h,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortNumber(u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PortNumberParser;

impl<'a> ArgParser<'a> for PortNumberParser {
    type Output = PortNumber;

    fn parse(
        &self,
        obj: &'a micropython_rs::obj::Obj,
    ) -> Result<Self::Output, argparse::ParseError> {
        IntParser::new(1..=21).parse(obj).map(|int| PortNumber(int))
    }
}

impl DefaultParser<'_> for PortNumber {
    type Parser = PortNumberParser;
}

impl PortNumber {
    pub const fn new(number: u8) -> Result<Self, ()> {
        if number >= 1 && number <= 21 {
            Ok(Self(number))
        } else {
            Err(())
        }
    }

    pub fn from_i32(number: i32) -> Result<Self, ()> {
        number.try_into().map_err(|_| ()).and_then(Self::new)
    }

    pub const fn number(self) -> u8 {
        self.0
    }
}

static REGISTRIES: LazyLock<Devices> = LazyLock::new(|| Devices::new().unwrap());

pub fn lock_port<D, I>(port: PortNumber, init: I) -> RegistryGuard<'static, SmartPort, D>
where
    D: PortDevice<SmartPort>,
    I: FnOnce(SmartPort) -> D,
{
    REGISTRIES.registry_by_port(port).lock(init)
}

pub fn lock_controller(id: ControllerId) -> ControllerGuard {
    match id {
        ControllerId::Primary => REGISTRIES.primary_controller.lock(|c| c),
        ControllerId::Partner => REGISTRIES.partner_controller.lock(|c| c),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdiPortNumber {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl std::fmt::Display for AdiPortNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => f.write_char('a'),
            Self::B => f.write_char('b'),
            Self::C => f.write_char('c'),
            Self::D => f.write_char('d'),
            Self::E => f.write_char('e'),
            Self::F => f.write_char('f'),
            Self::G => f.write_char('g'),
            Self::H => f.write_char('h'),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AdiPortNumberParser;

impl<'a> ArgParser<'a> for AdiPortNumberParser {
    type Output = AdiPortNumber;

    fn parse(&self, obj: &'a micropython_rs::obj::Obj) -> Result<Self::Output, ParseError> {
        let str = StrParser.parse(obj)?;
        if str.len() != 1 {
            return Err(ParseError::ValueError {
                mk_msg: Box::new(|arg| {
                    error_msg!("{arg}: adi port must only consist of one letter ('a' through 'h')")
                }),
            });
        }

        Ok(match str.chars().next().unwrap().to_ascii_lowercase() {
            'a' => AdiPortNumber::A,
            'b' => AdiPortNumber::B,
            'c' => AdiPortNumber::C,
            'd' => AdiPortNumber::D,
            'e' => AdiPortNumber::E,
            'f' => AdiPortNumber::F,
            'g' => AdiPortNumber::G,
            'h' => AdiPortNumber::H,
            _ => {
                return Err(ParseError::ValueError {
                    mk_msg: Box::new(|arg| {
                        error_msg!("{arg}: adi port must be a letter from 'a' through 'h'")
                    }),
                });
            }
        })
    }
}

impl DefaultParser<'_> for AdiPortNumber {
    type Parser = AdiPortNumberParser;
}

pub fn try_lock_adi_port(port: AdiPortNumber) -> Result<AdiPort, DeviceOccupiedError> {
    REGISTRIES.adi_registry_by_port(port).try_lock()
}

pub fn lock_display() -> MutexGuard<'static, Display> {
    REGISTRIES.display.lock().unwrap()
}
