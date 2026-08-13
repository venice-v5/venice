use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::analog::AdiAnalogIn;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Analog Input
///
/// # Overview
///
/// Unlike digital ADI devices which can only report a "high" or "low" state, analog ADI devices
/// may report a wide range of values spanning 0-5 volts. These analog voltages readings are then
/// converted into a digital values using the internal Analog-to-Digital Converter (ADC) in the V5
/// Brain. The Brain measures analog input using 12-bit values ranging from 0 (0V) to 4095 (5V).
///
/// Analog Input over ADI
///
/// Measures the voltage coming into an ADI port via a 12-bit ADC.
#[class(qstr!(AdiAnalogIn))]
pub struct AdiAnalogInObj {
    base: ObjBase,
    analog: AdiAnalogIn,
}

#[class_methods]
impl AdiAnalogInObj {
    /// The maximum 12-bit analog value returned by the internal analog-to-digital converters on the
    /// Brain.
    #[constant]
    const ADC_MAX_VALUE: i32 = vexide_devices::adi::analog::ADC_MAX_VALUE as i32;

    /// Configures an ADI port to measure analog input, returning an `AdiAnalogIn`.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort, /) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?.commit()?;
        Ok(Self {
            base: ty.into(),
            analog: AdiAnalogIn::new(port),
        })
    }

    /// Reads an analog input channel, returning the 12-bit value (0-4095).
    ///
    /// # Sensor Compatibility
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode. The
    /// meaning of the returned value varies depending on the sensor attached.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_value(&self) -> Result<i32, Exception> {
        Ok(self.analog.value()? as i32)
    }

    /// Reads an analog input channel and returns the calculated voltage input (0-5V).
    ///
    /// # Precision
    ///
    /// This function has a precision of `5.0/4095.0` volts, as ADC reports 12-bit voltage data on a
    /// scale of 0-4095.
    ///
    /// # Sensor Compatibility
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode. The
    /// meaning of the returned value varies depending on the sensor attached.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_voltage(&self) -> Result<f32, Exception> {
        Ok(self.analog.voltage()? as f32)
    }
}
