use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::value_error,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::{AdiPort, range_finder::AdiRangeFinder};

use crate::modvenice::{
    Exception,
    adi::{adi_port_name, expander::AdiPortParser, expander_index},
};

#[class(qstr!(AdiRangeFinder))]
#[repr(C)]
pub struct AdiRangeFinderObj {
    base: ObjBase,
    range_finder: AdiRangeFinder,
}

fn check_ports(output_port: &AdiPort, input_port: &AdiPort) -> Result<(), Exception> {
    let output_number = output_port.number();
    let input_number = input_port.number();

    // Input and output must be plugged into the same ADI expander.
    if expander_index(input_port.expander_number()) != expander_index(input_port.expander_number())
    {
        Err(value_error(error_msg!(
            "The specified output and input ports belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
            output_port.expander_number(),
            input_port.expander_number(),
        )))?;
    }

    // Output must be on an odd indexed port (A, C, E, G).
    if output_number.is_multiple_of(2) {
        Err(value_error(error_msg!(
            "The output ADI port provided (`{}`) was not odd numbered (A, C, E, G).",
            adi_port_name(output_number),
        )))?;
    }

    // Input must be directly next to top on the higher port index.
    if input_number != output_number + 1 {
        Err(value_error(error_msg!(
            "The input ADI port provided (`{}`) was not directly above the output port (`{}`). Instead, it should be port `{}`.",
            adi_port_name(input_number),
            adi_port_name(output_number),
            adi_port_name(output_number + 1),
        )))?;
    }

    Ok(())
}

#[class_methods]
impl AdiRangeFinderObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);

        let input_port = reader.next_positional_with(AdiPortParser)?;
        let output_port = reader.next_positional_with(AdiPortParser)?;
        check_ports(&output_port, &input_port)?;

        Ok(Self {
            base: ty.into(),
            range_finder: AdiRangeFinder::new(output_port, input_port),
        })
    }

    #[method]
    fn get_distance(&self) -> Result<Option<i32>, Exception> {
        Ok(self.range_finder.distance()?.map(i32::from))
    }
}
