use std::{
    cell::{Ref, RefCell, RefMut},
    sync::{Mutex, MutexGuard},
};

use micropython_rs::{except::value_error, init::token};
use thiserror::Error;
use vexide_devices::{controller::Controller, smart::SmartPort};

pub trait PortDevice<P> {
    fn take_port(self) -> P;
}

enum RegistryDevice<P> {
    Available(P),
    Occupied,
}

pub struct Registry<P> {
    device: Mutex<RegistryDevice<P>>,
}

struct ActiveRegistryGuard<'a, P, D>
where
    D: PortDevice<P>,
{
    device: D,
    guard: MutexGuard<'a, RegistryDevice<P>>,
}

pub struct UpgradeGuard<'a, P, D> {
    device: D,
    guard: MutexGuard<'a, RegistryDevice<P>>,
}

#[must_use]
pub struct RegistryGuard<'a, P, D>
where
    D: PortDevice<P>,
{
    guard: RefCell<Option<ActiveRegistryGuard<'a, P, D>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("device already freed")]
pub struct DeviceFreedError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("device occupied")]
pub struct DeviceOccupiedError;

impl<P> RegistryDevice<P> {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Occupied)
    }
}

impl<P> Registry<P> {
    pub const fn new(port: P) -> Self {
        Self {
            device: Mutex::new(RegistryDevice::Available(port)),
        }
    }

    pub fn try_lock<'a, D, I>(
        &'a self,
        init: I,
    ) -> Result<RegistryGuard<'a, P, D>, DeviceOccupiedError>
    where
        D: PortDevice<P>,
        I: FnOnce(P) -> D,
    {
        self.device
            .try_lock()
            .map(|mut registry_device| match registry_device.take() {
                RegistryDevice::Available(port) => RegistryGuard {
                    guard: RefCell::new(Some(ActiveRegistryGuard {
                        device: init(port),
                        guard: registry_device,
                    })),
                },
                RegistryDevice::Occupied => panic!("registry guard not dropped"),
            })
            .map_err(|_| DeviceOccupiedError)
    }

    pub fn lock<'a, D, I>(&'a self, init: I) -> RegistryGuard<'a, P, D>
    where
        D: PortDevice<P>,
        I: FnOnce(P) -> D,
    {
        self.try_lock(init)
            .unwrap_or_else(|_| value_error(c"port occupied").raise(token()))
    }
}

impl<'a, P, D> RegistryGuard<'a, P, D>
where
    D: PortDevice<P>,
{
    pub fn try_borrow<'b>(&'b self) -> Result<Ref<'b, D>, DeviceFreedError> {
        Ref::filter_map(self.guard.borrow(), |guard| {
            guard.as_ref().map(|guard| &guard.device)
        })
        .map_err(|_| DeviceFreedError)
    }

    pub fn try_borrow_mut<'b>(&'b self) -> Result<RefMut<'b, D>, DeviceFreedError> {
        RefMut::filter_map(self.guard.borrow_mut(), |guard| {
            guard.as_mut().map(|guard| &mut guard.device)
        })
        .map_err(|_| DeviceFreedError)
    }

    pub fn borrow<'b>(&'b self) -> Ref<'b, D> {
        self.try_borrow()
            .unwrap_or_else(|_| value_error(c"attempt to use device after free").raise(token()))
    }

    pub fn borrow_mut<'b>(&'b self) -> RefMut<'b, D> {
        self.try_borrow_mut()
            .unwrap_or_else(|_| value_error(c"attempt to use device after free").raise(token()))
    }

    pub fn start_upgrade(mut self) -> Result<UpgradeGuard<'a, P, D>, DeviceFreedError> {
        let guard = std::mem::replace(self.guard.get_mut(), None);
        match guard {
            Some(guard) => Ok(UpgradeGuard {
                device: guard.device,
                guard: guard.guard,
            }),
            None => Err(DeviceFreedError),
        }
    }

    pub fn finish_upgrade(upgrade: UpgradeGuard<'a, P, D>) -> Self {
        Self {
            guard: RefCell::new(Some(ActiveRegistryGuard {
                device: upgrade.device,
                guard: upgrade.guard,
            })),
        }
    }

    pub fn free(&self) -> Result<(), DeviceFreedError> {
        let guard = self.guard.replace(None);
        match guard {
            Some(mut guard) => {
                *guard.guard = RegistryDevice::Available(guard.device.take_port());
                Ok(())
            }
            None => Err(DeviceFreedError),
        }
    }

    pub fn free_or_raise(&self) {
        self.free()
            .unwrap_or_else(|_| value_error(c"attempt to free device twice").raise(token()))
    }
}

impl<'a, P, D> UpgradeGuard<'a, P, D> {
    pub fn map<E, F>(self, f: F) -> UpgradeGuard<'a, P, E>
    where
        F: FnOnce(D) -> E,
    {
        UpgradeGuard {
            device: f(self.device),
            guard: self.guard,
        }
    }

    pub fn as_mut(&mut self) -> &mut D {
        &mut self.device
    }
}

impl<'a, P, D> Drop for RegistryGuard<'a, P, D>
where
    D: PortDevice<P>,
{
    fn drop(&mut self) {
        let guard = self.guard.get_mut().take();
        if let Some(mut guard) = guard {
            *guard.guard = RegistryDevice::Available(guard.device.take_port());
        }
    }
}

pub type SmartRegistry = Registry<SmartPort>;
pub type SmartGuard<D> = RegistryGuard<'static, SmartPort, D>;

mod impls {
    use vexide_devices::smart::{
        SmartPort, ai_vision::AiVisionSensor, distance::DistanceSensor,
        electromagnet::Electromagnet, expander::AdiExpander, gps::GpsSensor, imu::InertialSensor,
        link::RadioLink, motor::Motor, optical::OpticalSensor, rotation::RotationSensor,
        serial::SerialPort, vision::VisionSensor,
    };

    use super::PortDevice;

    macro_rules! impl_device {
        ($port:ty, $($device:ty),*) => {
            $(
                impl PortDevice<$port> for $device {
                    fn take_port(self) -> $port {
                        self.into()
                    }
                }
            )*
        };
    }

    impl_device!(
        SmartPort,
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
        OpticalSensor,
        SmartPort
    );
}

impl PortDevice<Controller> for Controller {
    fn take_port(self) -> Controller {
        self
    }
}

pub type ControllerRegistry = Registry<Controller>;
pub type ControllerGuard = RegistryGuard<'static, Controller, Controller>;
