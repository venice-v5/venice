use std::any::Any;

use vexide_devices::smart::SmartPort;

mod impls;
mod registry;

pub trait Device: Any {
    fn take_port(self) -> SmartPort
    where
        Self: Sized;
}

pub use registry::Registry;
