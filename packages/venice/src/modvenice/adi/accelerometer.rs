use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::adi::accelerometer::{AdiAccelerometer, Sensitivity};

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

#[class(qstr!(AdiAccelerometerSensitivity))]
#[repr(C)]
pub struct SensitivityObj {
    base: ObjBase,
    sensitivity: Sensitivity,
}

#[class_methods]
impl SensitivityObj {
    const fn new(sensitivity: Sensitivity) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            sensitivity,
        }
    }

    #[constant]
    pub const LOW: &Self = &Self::new(Sensitivity::Low);
    #[constant]
    pub const HIGH: &Self = &Self::new(Sensitivity::High);
}

#[class(qstr!(AdiAccelerometer))]
#[repr(C)]
pub struct AdiAccelerometerObj {
    base: ObjBase,
    accelerometer: AdiAccelerometer,
}

#[class_methods]
impl AdiAccelerometerObj {
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
        let sensitivity = reader.next_positional::<&SensitivityObj>()?; // TODO: default value?
        Ok(Self {
            base: ty.into(),
            accelerometer: AdiAccelerometer::new(port, sensitivity.sensitivity),
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "sensitivity" => Obj::from_static(match self.accelerometer.sensitivity() {
                Sensitivity::Low => SensitivityObj::LOW,
                Sensitivity::High => SensitivityObj::HIGH,
            }),
            "max_acceleration" => Obj::from_float(self.accelerometer.max_acceleration() as f32),
            _ => return,
        })
    }

    #[method]
    fn get_acceleration(&self) -> Result<f32, Exception> {
        Ok(self.accelerometer.acceleration()? as f32)
    }

    #[method]
    fn raw_acceleration(&self) -> Result<i32, Exception> {
        Ok(self.accelerometer.raw_acceleration()? as i32)
    }
}
