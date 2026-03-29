use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::analog::AdiAnalogIn;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

#[class(qstr!(AdiAnalogIn))]
pub struct AdiAnalogInObj {
    base: ObjBase,
    analog: AdiAnalogIn,
}

#[class_methods]
impl AdiAnalogInObj {
    #[constant]
    const ADC_MAX_VALUE: i32 = vexide_devices::adi::analog::ADC_MAX_VALUE as i32;

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
            analog: AdiAnalogIn::new(port),
        })
    }

    #[method]
    fn get_value(&self) -> Result<i32, Exception> {
        Ok(self.analog.value()? as i32)
    }

    #[method]
    fn get_voltage(&self) -> Result<f32, Exception> {
        Ok(self.analog.voltage()? as f32)
    }
}
