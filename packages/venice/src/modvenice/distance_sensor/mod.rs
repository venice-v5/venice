pub mod distance_object;

use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::distance::{DistanceObjectError, DistanceSensor};

use crate::{
    devices::{self},
    modvenice::{Exception, device_error, distance_sensor::distance_object::DistanceObjectObj},
    registry::SmartGuard,
};

#[class(qstr!(DistanceSensor))]
#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase,
    guard: SmartGuard<DistanceSensor>,
}

impl From<DistanceObjectError> for Exception {
    fn from(value: DistanceObjectError) -> Self {
        device_error(error_msg!("{value}"))
    }
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
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    #[method]
    fn get_object(&self) -> Result<Option<DistanceObjectObj>, Exception> {
        Ok(self.guard.borrow().object()?.map(DistanceObjectObj::new))
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
