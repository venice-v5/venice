use std::any::TypeId;

use vexide_devices::smart::SmartPort;

use crate::Device;

fn downcast_device<D: Device>(device: Box<dyn Device>) -> Result<Box<D>, Box<dyn Device>> {
    if device.type_id() == TypeId::of::<D>() {
        let raw = Box::into_raw(device);
        Ok(unsafe { Box::from_raw(raw as *mut D) })
    } else {
        Err(device)
    }
}

enum RegistryDevice {
    Port(SmartPort),
    Device {
        device: Box<dyn Device>,
        destructor: fn(Box<dyn Device>) -> SmartPort,
    },
    Occupied,
}

pub struct Registry {
    device: RegistryDevice,
}

impl RegistryDevice {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Occupied)
    }
}

impl Registry {
    pub const fn new(port: SmartPort) -> Self {
        Self {
            device: RegistryDevice::Port(port),
        }
    }

    pub fn with_device<D, F, I, R>(&mut self, f: F, init: I) -> R
    where
        D: Device,
        F: FnOnce(&mut D) -> R,
        I: FnOnce(SmartPort) -> D,
    {
        let (device, ret) = match self.device.take() {
            RegistryDevice::Port(port) => {
                let mut device = init(port);
                let ret = f(&mut device);
                (
                    RegistryDevice::Device {
                        device: Box::new(device),
                        destructor: |device| {
                            downcast_device::<D>(device)
                                .unwrap_or_else(|_| panic!("destructor called on invalid device"))
                                .take_port()
                        },
                    },
                    ret,
                )
            }
            RegistryDevice::Device { device, destructor } => match downcast_device::<D>(device) {
                Ok(mut device) => {
                    let ret = f(&mut device);
                    (
                        RegistryDevice::Device {
                            device: device as Box<dyn Device>,
                            destructor: destructor,
                        },
                        ret,
                    )
                }
                Err(device) => {
                    let port = destructor(device);
                    let mut device = init(port);
                    let ret = f(&mut device);
                    (
                        RegistryDevice::Device {
                            device: Box::new(device),
                            destructor: |device| {
                                downcast_device::<D>(device)
                                    .unwrap_or_else(|_| {
                                        panic!("destructor called on invalid device")
                                    })
                                    .take_port()
                            },
                        },
                        ret,
                    )
                }
            },
            RegistryDevice::Occupied => panic!(),
        };
        self.device = device;

        ret
    }
}
