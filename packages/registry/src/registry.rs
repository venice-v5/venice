use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

use vexide_devices::smart::SmartPort;

use crate::Device;

enum RegistryDevice {
    Port(SmartPort),
    Occupied,
}

pub struct Registry {
    device: Mutex<RegistryDevice>,
}

#[must_use]
pub struct RegistryGuard<'a, D: Device> {
    // only used in Drop implementation, should never be None in usage
    device: Option<D>,
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
        D: Device,
        I: FnOnce(SmartPort) -> D,
    {
        self.device
            .try_lock()
            .map(|mut registry_device| match registry_device.take() {
                RegistryDevice::Port(port) => RegistryGuard {
                    device: Some(init(port)),
                    registry_device: registry_device,
                },
                RegistryDevice::Occupied => panic!("registry guard not dropped"),
            })
            .map_err(|_| ())
    }
}

impl<'a, D: Device> Deref for RegistryGuard<'a, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        self.device.as_ref().unwrap()
    }
}

impl<'a, D: Device> DerefMut for RegistryGuard<'a, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.device.as_mut().unwrap()
    }
}

impl<'a, D: Device> Drop for RegistryGuard<'a, D> {
    fn drop(&mut self) {
        *self.registry_device = RegistryDevice::Port(self.device.take().unwrap().take_port())
    }
}
