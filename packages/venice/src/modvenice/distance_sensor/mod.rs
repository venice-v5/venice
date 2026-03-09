pub mod distance_object;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    except::raise_value_error,
    init::token,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::distance::DistanceSensor;

use crate::{
    devices::{self, PortNumber},
    modvenice::{distance_sensor::distance_object::DistanceObjectObj, raise_port_error},
    qstrgen::qstr,
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
    fn make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Self {
        let token = token();
        let mut reader = Args::new(n_pos, n_kw, args).reader(token);
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = PortNumber::from_i32(reader.next_positional())
            .unwrap_or_else(|_| raise_value_error(token, c"port number must be between 1 and 21"));

        let guard = devices::lock_port(port, DistanceSensor::new);

        DistanceSensorObj {
            base: ObjBase::new(ty),
            guard,
        }
    }

    #[method]
    fn status(&self) -> i32 {
        self.guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e)) as i32
    }

    #[method]
    fn object(&self) -> Option<DistanceObjectObj> {
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
