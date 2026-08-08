use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::adi::line_tracker::AdiLineTracker;

use crate::modvenice::{Exception, adi::expander::AdiPortParser};

/// ADI Line Tracker
///
/// Line trackers read the difference between a black line and a white surface. They can be used to
/// follow a marked path on the ground.
///
/// In the V5 ecosystem, line trackers can be used to determine whether a robot is on a white tape
/// line placed on the field. This can be used to determine where a robot is.
///
/// While line trackers can be used in other applications besides line following, they may not be as
/// reliable when not used in a controlled environment (like pointed down at a surface). For
/// example, they may not be reliable when pointed upward and used to detect objects because there
/// may be infrared light sources in the environment that could interfere with the sensor's
/// readings.
///
/// # Hardware Overview
///
/// A line tracker consists of an analog infrared light sensor and an infrared LED. It works by
/// illuminating a surface with infrared light; the sensor then picks up the reflected infrared
/// radiation and, based on its intensity, determines the reflectivity of the surface in question.
/// White surfaces will reflect more light than dark surfaces, resulting in their appearing brighter
/// to the sensor. This allows the sensor to detect a dark line on a white background, or a white
/// line on a dark background.
///
/// The Line Tracking Sensor is an analog sensor, and it internally measures values in the range of
/// 0 to 4095 from 0-5V. Darker objects reflect less light, and are indicated by higher numbers.
/// Lighter objects reflect more light, and are indicated by lower numbers.
///
/// Internally, the sensor is comprised of an EE-SB5 photomicrosensor manufactured
/// by Omron mounted in a red housing. The sensor has a standard sensing distance
/// of 5mm.
///
/// More information about the sensor can be found in the
/// [datasheet](https://omronfs.omron.com/en_US/ecb/products/pdf/en-ee_sb5.pdf).
///
/// # Effective Range
///
/// For best results when using the Line Tracking Sensors, it is best to mount the sensors between
/// 1/8 and 1/4 of an inch away from the surface it is measuring. It is also important to keep
/// lighting in the room consistent, so sensors' readings remain accurate.
///
/// Line Tracker
#[class(qstr!(AdiLineTracker))]
#[repr(C)]
pub struct AdiLineTrackerObj {
    base: ObjBase,
    tracker: AdiLineTracker,
}

#[class_methods]
impl AdiLineTrackerObj {
    /// Creates a line tracker on the given `port`.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     line_tracker = AdiLineTracker("B")
    ///     while True:
    ///         print("Reflectivity: {}%".format(line_tracker.get_reflectivity() * 100.0))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
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
            tracker: AdiLineTracker::new(port),
        })
    }

    /// Returns the reflectivity factor measured by the sensor. Higher numbers mean a more
    /// reflective object.
    ///
    /// This is returned as a value ranging from [0.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     line_tracker = AdiLineTracker("B")
    ///     while True:
    ///         print("Reflectivity: {}%".format(line_tracker.get_reflectivity() * 100.0))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_reflectivity(&self) -> Result<f32, Exception> {
        Ok(self.tracker.reflectivity()? as f32)
    }

    /// Returns the 12-bit reflectivity reading of the sensor.
    ///
    /// This is a raw 12-bit value from [0, 4095] representing the voltage level from 0-5V measured
    /// by the V5 Brain's ADC.
    ///
    /// A low number (less voltage) represents a **more** reflective object.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     line_tracker = AdiLineTracker("B")
    ///     while True:
    ///         print("Raw 12-bit reflectivity:", line_tracker.get_raw_reflectivity())
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn get_raw_reflectivity(&self) -> Result<i32, Exception> {
        Ok(self.tracker.raw_reflectivity()? as i32)
    }
}
