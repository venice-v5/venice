use std::{
    cell::{Ref, RefCell, RefMut},
    mem::forget,
    sync::{Mutex, MutexGuard},
};

use micropython_rs::{except::value_error, init::token};
use thiserror::Error;
use vexide_devices::{adi::AdiPort, controller::Controller, smart::SmartPort};

use crate::lifecycle::GenerationTracker;

pub trait PortDevice<P> {
    fn take_port(self) -> P;
}

enum RegistryDevice<P> {
    Available(P),
    Occupied,
}

pub struct Registry<P> {
    device: Mutex<RegistryDevice<P>>,
    generations: GenerationTracker,
}

struct ActiveRegistryGuard<'a, P, D>
where
    D: PortDevice<P>,
{
    device: D,
    guard: MutexGuard<'a, RegistryDevice<P>>,
    generations: &'a GenerationTracker,
    generation: u64,
}

pub struct UpgradeGuard<'a, P, D> {
    device: D,
    guard: MutexGuard<'a, RegistryDevice<P>>,
    generations: &'a GenerationTracker,
    generation: u64,
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
            generations: GenerationTracker::new(),
        }
    }

    pub fn is_generation_active(&self, generation: u64) -> bool {
        self.generations.is_active(generation)
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
                RegistryDevice::Available(port) => {
                    let generation = self.generations.activate();
                    RegistryGuard {
                        guard: RefCell::new(Some(ActiveRegistryGuard {
                            device: init(port),
                            guard: registry_device,
                            generations: &self.generations,
                            generation,
                        })),
                    }
                }
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

    pub fn generation(&self) -> Result<u64, DeviceFreedError> {
        self.guard
            .borrow()
            .as_ref()
            .map(|guard| guard.generation)
            .ok_or(DeviceFreedError)
    }

    pub fn start_upgrade(mut self) -> Result<UpgradeGuard<'a, P, D>, DeviceFreedError> {
        let guard = std::mem::replace(self.guard.get_mut(), None);
        match guard {
            Some(guard) => Ok(UpgradeGuard {
                device: guard.device,
                guard: guard.guard,
                generations: guard.generations,
                generation: guard.generation,
            }),
            None => Err(DeviceFreedError),
        }
    }

    pub fn finish_upgrade(upgrade: UpgradeGuard<'a, P, D>) -> Self {
        Self {
            guard: RefCell::new(Some(ActiveRegistryGuard {
                device: upgrade.device,
                guard: upgrade.guard,
                generations: upgrade.generations,
                generation: upgrade.generation,
            })),
        }
    }

    pub fn take(&self) -> Result<D, DeviceFreedError> {
        let guard = self.guard.replace(None);
        match guard {
            Some(guard) => {
                forget(guard.guard);
                Ok(guard.device)
            }
            None => Err(DeviceFreedError),
        }
    }

    pub fn free(&self) -> Result<(), DeviceFreedError> {
        let guard = self.guard.replace(None);
        match guard {
            Some(mut guard) => {
                guard.generations.deactivate(guard.generation);
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
            generations: self.generations,
            generation: self.generation,
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
            guard.generations.deactivate(guard.generation);
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

pub struct AdiRegistry {
    port: Mutex<Option<AdiPort>>,
}

impl AdiRegistry {
    pub const fn new(port: AdiPort) -> Self {
        Self {
            port: Mutex::new(Some(port)),
        }
    }

    pub fn is_available(&self) -> bool {
        self.port.lock().unwrap().is_some()
    }

    pub fn try_lock(&self) -> Result<AdiPort, DeviceOccupiedError> {
        self.port.lock().unwrap().take().ok_or(DeviceOccupiedError)
    }

    pub fn lock(&self) -> AdiPort {
        self.try_lock()
            .unwrap_or_else(|_| value_error(c"adi port occupied").raise(token()))
    }
}
