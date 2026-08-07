use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::light_sensor::AdiLightSensor;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Light Sensor
///
/// The Light Sensor measures the intensity of visible light with a photoresistor.
///
/// # Hardware Overview
///
/// Using a Cadmium Sulfoselenide photoconductive photocell (CdS), the light sensor is able to
/// adjust its resistance based on the amount of visible light shining on it.
///
/// The light sensor only measures light in the visible spectrum. It cannot detect infrared or
/// ultraviolet sources.
///
/// # Effective Range
///
/// Effective range is dependent on both the intensity of the source and the surrounding
/// environment. Darker ambient surroundings with a brighter source will result in a greater
/// effective range.
///
/// That being said, the sensor generally has a usable range of up to 6 feet, meaning it can
/// distinguish a light source from the surrounding ambient light at up to six feet away.
/// Measurements farther than this might cause the sensor to return inconclusive results or blend
/// into the ambient light.
///
/// Light Sensor
#[class(qstr!(AdiLightSensor))]
pub struct AdiLightSensorObj {
    base: ObjBase,
    sensor: AdiLightSensor,
}

#[class_methods]
impl AdiLightSensorObj {
    /// Creates a light sensor on the given `port`.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new light sensor on port A.
    /// sensor = AdiLightSensor("A")
    ///
    /// # Get the brightness value.
    /// print("Brightness value:", sensor.get_brightness())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort) -> None")]
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
            sensor: AdiLightSensor::new(port),
        })
    }

    /// Returns the brightness factor measured by the sensor. Higher numbers mean a brighter light
    /// source.
    ///
    /// This is returned as a value in the interval [0.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new light sensor on port A.
    /// sensor = AdiLightSensor("A")
    ///
    /// # Get the brightness value.
    /// print("Brightness value:", sensor.get_brightness())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.sensor.brightness()? as f32)
    }

    /// Returns the 12-bit brightness reading of the sensor.
    ///
    /// This is a raw 12-bit value in the interval [0, 4095] representing the voltage level from
    /// 0-5V measured by the V5 Brain's ADC.
    ///
    /// A low number (less voltage) represents a **brighter** light source.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new light sensor on port A.
    /// sensor = AdiLightSensor("A")
    ///
    /// # Get the brightness value.
    /// print("Raw 12-bit brightness value:", sensor.get_raw_brightness())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_raw_brightness(&self) -> Result<i32, Exception> {
        Ok(self.sensor.raw_brightness()? as i32)
    }
}
