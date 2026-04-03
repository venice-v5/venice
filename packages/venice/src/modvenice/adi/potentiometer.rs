use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::adi::potentiometer::{AdiPotentiometer, PotentiometerType};

use crate::modvenice::{Exception, adi::expander::AdiPortParser, units::rotation::RotationUnitObj};

#[class(qstr!(AdiPotentiometer))]
#[repr(C)]
pub struct AdiPotentiometerObj {
    base: ObjBase,
    potentiometer: AdiPotentiometer,
}

#[class(qstr!(PotentiometerType))]
#[repr(C)]
pub struct PotentiometerTypeObj {
    base: ObjBase,
    ty: PotentiometerType,
}

#[class_methods]
impl PotentiometerTypeObj {
    const fn new(ty: PotentiometerType) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            ty,
        }
    }

    #[constant]
    const LEGACY: &Self = &Self::new(PotentiometerType::Legacy);
    #[constant]
    const V2: &Self = &Self::new(PotentiometerType::V2);

    #[constant]
    const LEGACY_MAX_ANGLE_DEG: f32 = 250.0;
    #[constant]
    const V2_MAX_ANGLE_DEG: f32 = 333.0;

    #[method]
    fn get_max_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(self.ty.max_angle())
    }
}

#[class_methods]
impl AdiPotentiometerObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        let potentiometer_type = reader.next_positional::<&PotentiometerTypeObj>()?;

        Ok(Self {
            base: ty.into(),
            potentiometer: AdiPotentiometer::new(port, potentiometer_type.ty),
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "type" => Obj::from_static(match self.potentiometer.potentiometer_type() {
                PotentiometerType::Legacy => PotentiometerTypeObj::LEGACY,
                PotentiometerType::V2 => PotentiometerTypeObj::V2,
            }),
            _ => return,
        })
    }

    #[method]
    fn get_max_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(self.potentiometer.max_angle())
    }

    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.potentiometer.angle()?))
    }
}
