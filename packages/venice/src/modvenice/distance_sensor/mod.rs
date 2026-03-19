pub mod distance_object;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::distance::DistanceSensor;

use crate::{
    devices::{self},
    modvenice::{Exception, distance_sensor::distance_object::DistanceObjectObj, raise_port_error},
    registry::RegistryGuard,
};

#[class(qstr!(DistanceSensor))]
#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, DistanceSensor>,
}

#[class_methods]
impl DistanceSensorObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let guard = devices::lock_port(port, DistanceSensor::new);

        Ok(DistanceSensorObj {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
    fn get_status(&self) -> i32 {
        self.guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e)) as i32
    }

    #[method]
    fn get_object(&self) -> Option<DistanceObjectObj> {
        self.guard
            .borrow()
            .object()
            .unwrap_or_else(|e| raise_port_error!(e))
            .map(DistanceObjectObj::new)
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
