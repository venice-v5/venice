use std::{
    cell::{Ref, RefCell, RefMut},
    sync::{Mutex, MutexGuard},
};

use micropython_rs::{except::raise_value_error, init::token};
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

struct ActiveRegistryGuard<'a, D: PortDevice> {
    device: D,
    guard: MutexGuard<'a, RegistryDevice>,
}

#[must_use]
pub struct RegistryGuard<'a, D: PortDevice> {
    guard: RefCell<Option<ActiveRegistryGuard<'a, D>>>,
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
                    guard: RefCell::new(Some(ActiveRegistryGuard {
                        device: init(port),
                        guard: registry_device,
                    })),
                },
                RegistryDevice::Occupied => panic!("registry guard not dropped"),
            })
            .map_err(|_| ())
    }

    pub fn lock<'a, D, I>(&'a self, init: I) -> RegistryGuard<'a, D>
    where
        D: PortDevice,
        I: FnOnce(SmartPort) -> D,
    {
        self.try_lock(init)
            .unwrap_or_else(|_| raise_value_error(token().unwrap(), "port occupied"))
    }
}

impl<'a, D: PortDevice> RegistryGuard<'a, D> {
    pub fn try_borrow<'b>(&'b self) -> Result<Ref<'b, D>, ()> {
        Ref::filter_map(self.guard.borrow(), |guard| {
            guard.as_ref().map(|guard| &guard.device)
        })
        .map_err(|_| ())
    }

    pub fn try_borrow_mut<'b>(&'b self) -> Result<RefMut<'b, D>, ()> {
        RefMut::filter_map(self.guard.borrow_mut(), |guard| {
            guard.as_mut().map(|guard| &mut guard.device)
        })
        .map_err(|_| ())
    }

    pub fn borrow<'b>(&'b self) -> Ref<'b, D> {
        self.try_borrow().unwrap_or_else(|_| {
            raise_value_error(token().unwrap(), "attempt to use device after free")
        })
    }

    pub fn borrow_mut<'b>(&'b self) -> RefMut<'b, D> {
        self.try_borrow_mut().unwrap_or_else(|_| {
            raise_value_error(token().unwrap(), "attempt to use device after free")
        })
    }

    pub fn free(&self) -> Result<(), ()> {
        let guard = self.guard.replace(None);
        match guard {
            Some(mut guard) => {
                *guard.guard = RegistryDevice::Port(guard.device.take_port());
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn free_or_raise(&self) {
        self.free()
            .unwrap_or_else(|_| raise_value_error(token().unwrap(), "attempt to free device twice"))
    }
}

impl<'a, D: PortDevice> Drop for RegistryGuard<'a, D> {
    fn drop(&mut self) {
        let guard = std::mem::replace(self.guard.get_mut(), None);
        if let Some(mut guard) = guard {
            *guard.guard = RegistryDevice::Port(guard.device.take_port());
        }
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
