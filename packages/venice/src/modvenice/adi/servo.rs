use std::cell::RefCell;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::servo::AdiServo;

use crate::modvenice::{Exception, adi::expander::AdiPortParser, units::rotation::RotationUnitObj};

#[class(qstr!(AdiServo))]
pub struct AdiServoObj {
    base: ObjBase,
    servo: RefCell<AdiServo>,
}

#[class_methods]
impl AdiServoObj {
    #[constant]
    const MIN_POSITION_DEG: f32 = AdiServo::MIN_POSITION.as_degrees() as f32;
    #[constant]
    const MAX_POSITION_DEG: f32 = AdiServo::MAX_POSITION.as_degrees() as f32;

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
            servo: RefCell::new(AdiServo::new(port)),
        })
    }

    #[method]
    fn set_target(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .servo
            .borrow_mut()
            .set_target(unit.unit().float_to_angle(position))?)
    }

    #[method]
    fn set_raw_target(&self, pwm: i8) -> Result<(), Exception> {
        Ok(self.servo.borrow_mut().set_raw_target(pwm)?)
    }
}
