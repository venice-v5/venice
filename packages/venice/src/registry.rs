use std::{
    cell::{Ref, RefCell, RefMut},
    sync::{Mutex, MutexGuard},
};

use vexide_devices::smart::SmartPort;

pub trait PortDevice {
    fn take_port(self) -> SmartPort;
}

enum RegistryDevice {
    Port(SmartPort),
    Occupied,
}

pub struct Registry {
    device: Mutex<RegistryDevice>,
}

enum GuardDevice<D: PortDevice> {
    Active(RefCell<D>),
    Dropped,
}

#[must_use]
pub struct RegistryGuard<'a, D: PortDevice> {
    // only used in Drop implementation, should never be None in usage
    device: GuardDevice<D>,
    registry_device: MutexGuard<'a, RegistryDevice>,
}

impl RegistryDevice {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Occupied)
    }
}

impl Registry {
    pub const fn new(port: SmartPort) -> Self {
        Self {
            device: Mutex::new(RegistryDevice::Port(port)),
        }
    }

    pub fn try_lock<'a, D, I>(&'a self, init: I) -> Result<RegistryGuard<'a, D>, ()>
    where
        D: PortDevice,
        I: FnOnce(SmartPort) -> D,
    {
        self.device
            .try_lock()
            .map(|mut registry_device| match registry_device.take() {
                RegistryDevice::Port(port) => RegistryGuard {
                    device: GuardDevice::Active(RefCell::new(init(port))),
                    registry_device: registry_device,
                },
                RegistryDevice::Occupied => panic!("registry guard not dropped"),
            })
            .map_err(|_| ())
    }
}

impl<D: PortDevice> GuardDevice<D> {
    fn borrow<'a>(&'a self) -> Ref<'a, D> {
        match self {
            Self::Active(d) => d.borrow(),
            // should never happen
            Self::Dropped => panic!(),
        }
    }

    fn borrow_mut<'a>(&'a self) -> RefMut<'a, D> {
        match self {
            Self::Active(d) => d.borrow_mut(),
            // should never happen
            Self::Dropped => panic!(),
        }
    }

    fn get_mut(&mut self) -> &mut D {
        match self {
            Self::Active(d) => d.get_mut(),
            // should never happen
            Self::Dropped => panic!(),
        }
    }

    fn drop(&mut self) -> SmartPort {
        match std::mem::replace(self, Self::Dropped) {
            Self::Active(d) => d.into_inner().take_port(),
            // should never happen
            Self::Dropped => panic!("attempt to drop GuardDevice twice"),
        }
    }
}

impl<'a, D: PortDevice> RegistryGuard<'a, D> {
    pub fn borrow(&'a self) -> Ref<'a, D> {
        self.device.borrow()
    }

    pub fn borrow_mut(&'a self) -> RefMut<'a, D> {
        self.device.borrow_mut()
    }

    pub fn get_mut(&mut self) -> &mut D {
        self.device.get_mut()
    }
}

impl<'a, D: PortDevice> Drop for RegistryGuard<'a, D> {
    fn drop(&mut self) {
        *self.registry_device = RegistryDevice::Port(self.device.drop())
    }
}

mod impls {
    use vexide_devices::smart::{
        SmartPort, ai_vision::AiVisionSensor, distance::DistanceSensor,
        electromagnet::Electromagnet, expander::AdiExpander, gps::GpsSensor, imu::InertialSensor,
        link::RadioLink, motor::Motor, optical::OpticalSensor, rotation::RotationSensor,
        serial::SerialPort, vision::VisionSensor,
    };

    use super::PortDevice;

    macro_rules! impl_device {
        ($($device:ty),*) => {
            $(
                impl PortDevice for $device {
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
}
