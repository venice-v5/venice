use std::cell::RefCell;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::encoder::AdiEncoder;

use crate::modvenice::{Exception, adi::expander::AdiPortParser, units::rotation::RotationUnitObj};

#[class(qstr!(AdiEncoder))]
pub struct AdiEncoderObj {
    base: ObjBase,
    // vexide doesn't support non-const tpr values, so we have to set a tpr of 1 and manually make
    // tpr corrections
    // theta = (ticks * TAU) / tpr
    // ticks = (theta * tpr) / TAU
    encoder: RefCell<AdiEncoder<1>>,
    tpr: i32,
}

#[class_methods]
impl AdiEncoderObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 3).assert_nkw(0, 0);

        let top_port = reader.next_positional_with(AdiPortParser)?;
        let bottom_port = reader.next_positional_with(AdiPortParser)?;
        let tpr = reader.next_positional_or(360)?;

        Ok(Self {
            base: ty.into(),
            encoder: RefCell::new(AdiEncoder::new(top_port, bottom_port)),
            tpr,
        })
    }

    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let tick_turns = self.encoder.borrow().position()?; // ticks * TAU
        let position = tick_turns / self.tpr as f64; // (ticks * TAU) / tpr
        Ok(unit.unit().angle_to_float(position))
    }

    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position); // theta
        let ticks = angle * self.tpr as f64; // theta * TPR
        Ok(self.encoder.borrow_mut().set_position(ticks)?) // function internally divides by TAU
    }

    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        Ok(self.encoder.borrow_mut().reset_position()?)
    }
}
