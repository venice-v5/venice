use std::sync::LazyLock;

use vexide_devices::{controller::ControllerId, peripherals::Peripherals, smart::SmartPort};

use crate::registry::{ControllerGuard, ControllerRegistry, PortDevice, Registry, RegistryGuard};

pub struct Devices {
    pub primary_controller: ControllerRegistry,
    pub partner_controller: ControllerRegistry,

    pub port_1: Registry,
    pub port_2: Registry,
    pub port_3: Registry,
    pub port_4: Registry,
    pub port_5: Registry,
    pub port_6: Registry,
    pub port_7: Registry,
    pub port_8: Registry,
    pub port_9: Registry,
    pub port_10: Registry,
    pub port_11: Registry,
    pub port_12: Registry,
    pub port_13: Registry,
    pub port_14: Registry,
    pub port_15: Registry,
    pub port_16: Registry,
    pub port_17: Registry,
    pub port_18: Registry,
    pub port_19: Registry,
    pub port_20: Registry,
    pub port_21: Registry,
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
        })
    }

    fn registry_by_port(&self, port: PortNumber) -> &Registry {
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
        number.try_into().map_err(|_| ()).and_then(Self::new)
    }

    pub const fn number(self) -> u8 {
        self.0
    }
}

static REGISTRIES: LazyLock<Devices> = LazyLock::new(|| Devices::new().unwrap());

pub fn lock_port<D, I>(port: PortNumber, init: I) -> RegistryGuard<'static, D>
where
    D: PortDevice,
    I: FnOnce(SmartPort) -> D,
{
    REGISTRIES.registry_by_port(port).lock(init)
}

pub fn lock_controller(id: ControllerId) -> ControllerGuard<'static> {
    match id {
        ControllerId::Primary => REGISTRIES.primary_controller.lock(),
        ControllerId::Partner => REGISTRIES.partner_controller.lock(),
    }
}
