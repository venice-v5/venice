pub mod distance_object;

use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::distance::{DistanceObjectError, DistanceSensor};

use crate::{
    devices::{self},
    modvenice::{Exception, device_error, distance_sensor::distance_object::DistanceObjectObj},
    registry::SmartGuard,
};

/// Distance Sensor.
///
/// This class provides an interface to interact with the VEX V5 Distance Sensor, which uses a Class
/// 1 laser to measure the distance, object size classification, and relative velocity of a single
/// object.
///
/// # Hardware Overview
///
/// The sensor uses a narrow-beam Class 1 laser (similar to phone proximity sensors) for precise
/// detection. It measures distances from 20mm to 2000mm with varying accuracy (±15mm below 200mm, ±5%
/// above 200mm).
///
/// The sensor can classify detected objects by relative size, helping distinguish between walls and
/// field elements. It also measures the relative approach velocity between the sensor and target.
///
/// Due to the use of a laser, measurements are single-point and highly directional, meaning that
/// objects will only be detected when they are directly in front of the sensor's field of view.
///
/// Like all other Smart devices, VEXos will process sensor updates every 10mS.
#[class(qstr!(DistanceSensor))]
#[repr(C)]
pub struct DistanceSensorObj {
    base: ObjBase,
    guard: SmartGuard<DistanceSensor>,
}

impl From<DistanceObjectError> for Exception {
    fn from(value: DistanceObjectError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl DistanceSensorObj {
    /// Creates a new distance sensor from Smart Port `port`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = DistanceSensor(1)
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` isn't from 1 through 21 or is already in use.
    /// - `TypeError`: If `port` is not an integer.
    #[make_new]
    #[stub(sig = "(self, port: int) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let guard = devices::lock_port(port, DistanceSensor::new);

        Ok(DistanceSensorObj {
            base: ObjBase::new(ty),
            guard,
        })
    }

    /// Returns the internal status code of the distance sensor.
    ///
    /// The status code of the signature can tell you if the sensor is still initializing or if it is
    /// working correctly.
    ///
    /// - If the distance sensor is still initializing, the status code will be 0x00.
    /// - If it is done initializing and functioning correctly, the status code will be 0x82 or 0x86.
    ///
    /// # Examples
    ///
    /// A simple initialization state check:
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     sensor = DistanceSensor(1)
    ///     while True:
    ///         if sensor.get_status() == 0:
    ///             print("Sensor is still initializing")
    ///         else:
    ///             print("Sensor is ready")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// Printing the status code in binary format:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = DistanceSensor(1)
    /// print(f"Status: {sensor.get_status():b}")
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the sensor binding has been freed.
    /// - `DeviceError`: If no device is connected to the port or the wrong type of device is connected.
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    /// Attempts to detect an object, returning `None` if no object could be found.
    ///
    /// # Examples
    ///
    /// Measure object distance and velocity:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = DistanceSensor(1)
    /// object = sensor.get_object()
    /// if object is not None:
    ///     print("Object of size {}mm is moving at {}m/s".format(object.distance, object.velocity))
    /// ```
    ///
    /// Get object distance, but only with high confidence:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = DistanceSensor(1)
    /// object = sensor.get_object()
    /// distance = object.distance if object is not None and object.confidence > 0.8 else None
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the sensor binding has been freed.
    /// - `DeviceError`: If no sensor is connected to the port, the distance sensor is still initializing,
    ///   or it has an unknown status code.
    #[method]
    fn get_object(&self) -> Result<Option<DistanceObjectObj>, Exception> {
        Ok(self.guard.borrow().object()?.map(DistanceObjectObj::new))
    }

    /// Releases this sensor and frees its Smart Port lock.
    ///
    /// The object is unusable afterward, but its Smart Port can be assigned to another device.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the sensor has already been freed.
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
