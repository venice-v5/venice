use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::value_error,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::adi::range_finder::AdiRangeFinder;

use crate::modvenice::{
    Exception,
    adi::{
        adi_port_name,
        expander::{AdiPortParser, AdiPortSpec, commit_adi_port_pair},
        expander_index,
    },
};

/// ADI Ultrasonic Range Finder
///
/// The Ultrasonic Range Finder is a rangefinding device which uses ultrasonic sound to measure the
/// distance between the sensor and the object the sound is being reflected back from.
///
/// The Ultrasonic Range Finder is also known as a sonar sensor in VEXCode terminology.
///
/// # Hardware Overview
///
/// The Ultrasonic Rangefinder uses sound pulses to measure distance, in a similar way to how bats
/// or submarines find their way around. By emitting an 40KHz ultrasonic pulse for 250mS and timing
/// how long it takes to hear an echo, the Ultrasonic Rangefinder can accurately estimate how far
/// away an object in front of it is.
///
/// The equation used by the Ultrasonic Range Finder's to calculate its distance reading is `d = t *
/// 171.5` where "d" represents the distance between the sensor and the object found, "t" represents
/// the time it took for the sound wave to return to the sensor, and 171.5 is half the
/// speed of sound in `m/s`.
///
/// # Effective Range
///
/// The usable range of the Range Finder is between 1.5" (3.0cm) and 115" (300cm). When the sensor
/// attempts to measure an object at less than 1.5", the sound echos back too quickly for the sensor
/// to detect and much beyond 115" the intensity of the sound is too weak to detect.
///
/// Since the Ultrasonic Rangefinder relies on sound waves, surfaces that absorb or deflect sound
/// (such as cushioned surfaces or sharp angles) will limit the operating range of the sensor.
///
/// # Wiring
///
/// The sensor has two 3-Wire Cables. There is a black, red, and orange "Output" cable which pulses
/// power to a 40KHz speaker; and a black, red, and yellow "Input" cable which sends a signal back
/// from its high frequency microphone receiver.
///
/// When wiring the Ultrasonic Rangefinder to the, both wires must be plugged into adjacent ADI
/// ports. For the sensor to work properly, the "OUTPUT" wire must be in an odd-numbered slot
/// (A, C, E, G), and the "INPUT" wire must be in the higher slot next to the input wire.
///
/// Range Finder
///
/// Requires two ports - one for pinging (output), and one for listening for the response (input).
///
/// This output port ("ping") must be indexed directly below the input ("echo") port.
#[class(qstr!(AdiRangeFinder))]
#[repr(C)]
pub struct AdiRangeFinderObj {
    base: ObjBase,
    range_finder: AdiRangeFinder,
}

fn check_ports(output_port: AdiPortSpec<'_>, input_port: AdiPortSpec<'_>) -> Result<(), Exception> {
    let output_number = output_port.number();
    let input_number = input_port.number();

    // Input and output must be plugged into the same ADI expander.
    if expander_index(output_port.expander_number()) != expander_index(input_port.expander_number())
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
    /// Creates a new rangefinder sensor from an `input_port` and `output_port`.
    ///
    /// Both ports must belong to the same Brain or `AdiExpander`. The `output_port` must be A, C, E, or G,
    /// and `input_port` must be the next port above it.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     range_finder = AdiRangeFinder("B", "A")
    ///     while True:
    ///         distance = range_finder.get_distance()
    ///         if distance is not None:
    ///             print("Distance: {} cm".format(distance))
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If either port is invalid or occupied, the ports belong to different ADI
    ///   expanders, `output_port` is not A, C, E, or G, or `input_port` is not directly above it.
    #[make_new]
    #[stub(
        sig = "(self, input_port: str | AdiExpanderPort, output_port: str | AdiExpanderPort, /) -> None"
    )]
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
        check_ports(output_port, input_port)?;
        let (input_port, output_port) = commit_adi_port_pair(input_port, output_port)?;

        Ok(Self {
            base: ty.into(),
            range_finder: AdiRangeFinder::new(output_port, input_port),
        })
    }

    /// Returns the distance reading of the rangefinder sensor in centimeters, or `None` if the
    /// sensor was unable to find an object in range.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     range_finder = AdiRangeFinder("B", "A")
    ///     while True:
    ///         distance = range_finder.get_distance()
    ///         if distance is not None:
    ///             print("Distance: {} cm".format(distance))
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_distance(&self) -> Result<Option<i32>, Exception> {
        Ok(self.range_finder.distance()?.map(i32::from))
    }
}
