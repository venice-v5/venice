use std::cell::RefCell;

use argparse::{Args, error_msg};
use micropython_rs::{
    class, class_methods,
    except::value_error,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::{AdiPort, encoder::AdiEncoder};

use crate::modvenice::{
    Exception,
    adi::{adi_port_name, expander::AdiPortParser, expander_index},
    units::rotation::RotationUnitObj,
};

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

fn check_ports(top_port: &AdiPort, bottom_port: &AdiPort) -> Result<(), Exception> {
    if expander_index(top_port.expander_number()) != expander_index(bottom_port.expander_number()) {
        Err(value_error(error_msg!(
            "The specified top and bottom ports belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
            top_port.expander_number(),
            bottom_port.expander_number(),
        )))?;
    }

    let top_number = top_port.number();
    let bottom_number = bottom_port.number();
    let valid_combo = if top_number.is_multiple_of(2) {
        bottom_number == top_number - 1
    } else {
        bottom_number == top_number + 1
    };

    if !valid_combo {
        Err(value_error(error_msg!(
            "Encoder ports must be placed directly next to each other and in some combination of AB, CD, EF, GH, or BA, CD, EF, HG. (Got `{}{}`)",
            adi_port_name(top_number),
            adi_port_name(bottom_number),
        )))?;
    }

    Ok(())
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
        check_ports(&top_port, &bottom_port)?;
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
