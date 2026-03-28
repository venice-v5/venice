use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::line_tracker::AdiLineTracker;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

#[class(qstr!(AdiLineTracker))]
#[repr(C)]
pub struct AdiLineTrackerObj {
    base: ObjBase,
    tracker: AdiLineTracker,
}

#[class_methods]
impl AdiLineTrackerObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        Ok(Self {
            base: ty.into(),
            tracker: AdiLineTracker::new(port),
        })
    }

    #[method]
    fn get_reflectivity(&self) -> Result<f32, Exception> {
        Ok(self.tracker.reflectivity()? as f32)
    }

    #[method]
    fn get_raw_reflectivity(&self) -> Result<i32, Exception> {
        Ok(self.tracker.raw_reflectivity()? as i32)
    }
}
