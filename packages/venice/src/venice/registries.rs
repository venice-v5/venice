use std::cell::RefCell;

use vex_registry::{Device, Registry};
use vexide_devices::{peripherals::Peripherals, smart::SmartPort};

pub struct Registries {
    pub port_1: RefCell<Registry>,
    pub port_2: RefCell<Registry>,
    pub port_3: RefCell<Registry>,
    pub port_4: RefCell<Registry>,
    pub port_5: RefCell<Registry>,
    pub port_6: RefCell<Registry>,
    pub port_7: RefCell<Registry>,
    pub port_8: RefCell<Registry>,
    pub port_9: RefCell<Registry>,
    pub port_10: RefCell<Registry>,
    pub port_11: RefCell<Registry>,
    pub port_12: RefCell<Registry>,
    pub port_13: RefCell<Registry>,
    pub port_14: RefCell<Registry>,
    pub port_15: RefCell<Registry>,
    pub port_16: RefCell<Registry>,
    pub port_17: RefCell<Registry>,
    pub port_18: RefCell<Registry>,
    pub port_19: RefCell<Registry>,
    pub port_20: RefCell<Registry>,
    pub port_21: RefCell<Registry>,
}

impl Registries {
    fn new() -> Option<Self> {
        Peripherals::take().map(|peris| Self {
            port_1: RefCell::new(Registry::new(peris.port_1)),
            port_2: RefCell::new(Registry::new(peris.port_2)),
            port_3: RefCell::new(Registry::new(peris.port_3)),
            port_4: RefCell::new(Registry::new(peris.port_4)),
            port_5: RefCell::new(Registry::new(peris.port_5)),
            port_6: RefCell::new(Registry::new(peris.port_6)),
            port_7: RefCell::new(Registry::new(peris.port_7)),
            port_8: RefCell::new(Registry::new(peris.port_8)),
            port_9: RefCell::new(Registry::new(peris.port_9)),
            port_10: RefCell::new(Registry::new(peris.port_10)),
            port_11: RefCell::new(Registry::new(peris.port_11)),
            port_12: RefCell::new(Registry::new(peris.port_12)),
            port_13: RefCell::new(Registry::new(peris.port_13)),
            port_14: RefCell::new(Registry::new(peris.port_14)),
            port_15: RefCell::new(Registry::new(peris.port_15)),
            port_16: RefCell::new(Registry::new(peris.port_16)),
            port_17: RefCell::new(Registry::new(peris.port_17)),
            port_18: RefCell::new(Registry::new(peris.port_18)),
            port_19: RefCell::new(Registry::new(peris.port_19)),
            port_20: RefCell::new(Registry::new(peris.port_20)),
            port_21: RefCell::new(Registry::new(peris.port_21)),
        })
    }

    fn registry_by_port(&self, port: PortNumber) -> &RefCell<Registry> {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortNumber(u8);

impl PortNumber {
    pub const fn new(number: u8) -> Result<Self, ()> {
        if number >= 1 && number <= 21 {
            Ok(Self(number))
        } else {
            Err(())
        }
    }

    pub fn from_i32(number: i32) -> Result<Self, ()> {
        number
            .try_into()
            .map_err(|_| ())
            .and_then(|number| Self::new(number))
    }

    pub const fn number(self) -> u8 {
        self.0
    }
}

pub fn with_port<D, F, I, R>(port: PortNumber, f: F, init: I) -> R
where
    D: Device,
    F: FnOnce(&mut D) -> R,
    I: FnOnce(SmartPort) -> D,
{
    thread_local! {
        static REGISTRIES: Registries = Registries::new().unwrap_or_else(|| panic!("registries can only be accessed from the main thread"));
    }

    REGISTRIES.with(|registries| registries.registry_by_port(port).borrow_mut().with(f, init))
}
