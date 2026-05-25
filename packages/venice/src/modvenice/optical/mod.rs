pub mod gesture;
pub mod rgb;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::optical::OpticalSensor;

use crate::{
    devices,
    modvenice::{
        Exception,
        optical::{
            gesture::GestureObj,
            rgb::{OpticalRawObj, OpticalRgbObj},
        },
        units::time::TimeUnitObj,
    },
    registry::SmartGuard,
};

/// An optical sensor plugged into a Smart Port.
///
/// This class provides an interface to interact with the V5 Optical Sensor, which combines ambient
/// light sensing, color detection, proximity measurement, and gesture recognition capabilities.
///
/// # Hardware Overview
///
/// The optical sensor provides multi-modal optical sensing with an integrated white LED for
/// low-light operation.
///
/// ## Color Detection
///
/// Color data reported as RGB, HSV, and grayscale data, with optimal performance at distances under
/// 100mm. The proximity sensing uses reflected light intensity, making readings dependent on both
/// ambient lighting and target reflectivity.
///
/// ## Gesture Detection
///
/// The optical sensor can detect four distinct motions (up, down, left, right) of objects passing
/// over the sensor.
#[class(qstr!(OpticalSensor))]
#[repr(C)]
pub struct OpticalSensorObj {
    base: ObjBase,
    guard: SmartGuard<OpticalSensor>,
}

#[class_methods]
impl OpticalSensorObj {
    /// The smallest integration time you can set on an optical sensor, in milliseconds.
    ///
    /// Source: <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9>
    #[constant]
    const MIN_INTEGRATION_TIME_MS: i32 = OpticalSensor::MIN_INTEGRATION_TIME.as_millis() as i32;

    /// The largest integration time you can set on an optical sensor, in milliseconds.
    ///
    /// Source: <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9>
    #[constant]
    const MAX_INTEGRATION_TIME_MS: i32 = OpticalSensor::MAX_INTEGRATION_TIME.as_millis() as i32;

    /// The interval that gesture detection through `OpticalSensor.get_last_gesture` provides new
    /// data at.
    #[constant]
    const GESTURE_UPDATE_INTERVAL_MS: i32 =
        OpticalSensor::GESTURE_UPDATE_INTERVAL.as_millis() as i32;

    /// Creates a new optical sensor.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    /// ```
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

        Ok(OpticalSensorObj {
            base: ObjBase::new(ty),
            guard: devices::lock_port(port, |p| OpticalSensor::new(p)),
        })
    }

    /// Returns the detected color's hue.
    ///
    /// Hue has a range of `0` to `359.999`.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// hue = sensor.get_hue()
    /// print(f"Detected color hue: {hue:.1}°")
    /// ```
    #[method]
    fn get_hue(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().hue()? as f32)
    }

    /// Returns the detected color's saturation.
    ///
    /// Saturation has a range `0` to `1.0`.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// saturation = sensor.get_saturation()
    /// print(f"Color saturation: {saturation:.0%}")
    ///
    /// # Check if color is muted or vibrant
    /// if saturation < 0.5:
    ///     print("Muted color detected")
    /// else:
    ///     print("Vibrant color detected")
    /// ```
    #[method]
    fn get_saturation(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().saturation()? as f32)
    }

    /// Returns the detected color's brightness.
    ///
    /// Brightness values range from `0` to `1.0`.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// brightness = sensor.get_brightness()
    /// print(f"Color brightness: {brightness:.0%}")
    ///
    /// # Check if color is dark or bright
    /// if brightness < 0.3:
    ///     print("Dark color detected")
    /// elif brightness > 0.7:
    ///     print("Bright color detected")
    /// else:
    ///     print("Medium brightness color detected")
    /// ```
    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().brightness()? as f32)
    }

    /// Returns an analog proximity value from `0` to `1.0`.
    ///
    /// A reading of 1.0 indicates that the object is close to the sensor, while 0.0 indicates that
    /// no object is detected in range of the sensor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// # Monitor proximity with thresholds
    /// prox = sensor.get_proximity()
    /// if prox > 0.8:
    ///     print("Object very close!")
    /// elif prox > 0.5:
    ///     print("Object nearby")
    /// elif prox > 0.2:
    ///     print("Object detected")
    /// else:
    ///     print("No object in range")
    /// ```
    #[method]
    fn get_proximity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().proximity()? as f32)
    }

    /// Returns the processed RGB color data from the sensor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// rgb = sensor.get_color()
    /// print(f"Color reading: R={rgb.r}, G={rgb.g}, B={rgb.b}")
    ///
    /// # Example: Check if object is primarily red
    /// # Note that you should probably use `OpticalSensor.get_hue` instead for this
    /// if rgb.r > rgb.g && rgb.r > rgb.b:
    ///     print("Object is primarily red!")
    /// ```
    #[method]
    fn get_color(&self) -> Result<OpticalRgbObj, Exception> {
        Ok(OpticalRgbObj::new(self.guard.borrow().color()?))
    }

    /// Returns the raw, unprocessed RGBC color data from the sensor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    #[method]
    fn get_raw_color(&self) -> Result<OpticalRawObj, Exception> {
        Ok(OpticalRawObj::new(self.guard.borrow().raw_color()?))
    }

    /// Returns the most recent gesture data from the sensor, or `None` if no gesture was detected.
    ///
    /// Gesture data updates every 500 milliseconds.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// async def main():
    ///     while True:
    ///         gesture = sensor.get_last_gesture()
    ///         if direction:
    ///             print(f"Direction: {gesture.direction}")
    ///
    ///         await vasyncio.Sleep(25, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_last_gesture(&self) -> Result<Option<GestureObj>, Exception> {
        Ok(self.guard.borrow().last_gesture()?.map(GestureObj::new))
    }

    /// Returns the intensity/brightness of the sensor's LED indicator as a number from [0.0-1.0].
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// brightness = sensor.get_led_brightness()
    /// print(f"LED brightness: {brightness:.1%}")
    /// ```
    #[method]
    fn get_led_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().led_brightness()? as f32)
    }

    /// Set the intensity of (intensity/brightness) of the sensor's LED indicator.
    ///
    /// Intensity is expressed as a number from [0.0, 1.0].
    ///
    /// # Python
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// async def main():
    ///     for _ in range(3):
    ///         # Turn LED on
    ///         try:
    ///             sensor.set_led_brightness(1)
    ///         except DeviceError as e:
    ///             print(f"Failed to turn LED on: {e}")
    ///
    ///         await vasyncio.Sleep(250, MILLIS)
    ///
    ///         # Turn LED off
    ///         try:
    ///             sensor.set_led_brightness(0)
    ///         except DeviceError as e:
    ///             print(f"Failed to turn LED off: {e}")
    ///
    ///         await vasyncio.Sleep(250, MILLIS)
    /// ```
    #[method]
    fn set_led_brightness(&self, brightness: f32) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_led_brightness(brightness as f64)?)
    }

    /// Returns integration time of the optical sensor in milliseconds, with minimum time being 3ms
    /// and the maximum time being 712ms.
    ///
    /// The default integration time for the sensor is 103mS, unless otherwise set with
    /// `OpticalSensor.set_integration_time`.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// # Set integration time to 50 milliseconds.
    /// sensor.set_integration_time(50, MILLIS)
    ///
    /// # Log out the new integration time.
    /// integration_time = sensor.get_integration_time(MILLIS)
    /// print(f"Integration time: {integration_time}ms")
    /// ```
    #[method]
    fn get_integration_time(&self, unit: &TimeUnitObj) -> Result<f32, Exception> {
        Ok(unit
            .unit()
            .dur_to_float(self.guard.borrow().integration_time()?))
    }

    /// Set the integration time of the optical sensor.
    ///
    /// Lower integration time results in faster update rates with lower accuracy due to less
    /// available light being read by the sensor.
    ///
    /// The `time` value must be a duration between 3 and 712 milliseconds. If the integration
    /// time is out of this range, it will be clamped to fit inside it. See
    /// <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9> for more information.
    ///
    /// The default integration time for the sensor is 103mS, unless otherwise set.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// # Set integration time to 50 milliseconds.
    /// sensor.set_integration_time(50, MILLIS)
    /// ```
    #[method]
    fn set_integration_time(&self, time: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_integration_time(unit.unit().float_to_dur(time))?)
    }

    /// Returns the internal status code of the optical sensor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = OpticalSensor(1)
    ///
    /// status = sensor.get_status()
    /// print(f"Status: {status:b}")
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32) // should be OK to cast, the bits themselves stay the same
    }

    /// Release this device and free its Smart Port lock. This binding will become unusable after
    /// this call, but you can reuse the underlying Smart Port number in a new device.
    ///
    /// Any attempts to use this device after freeing will result in a `ValueError` being raised.
    ///
    /// # Raises
    ///
    /// `ValueError`: If the device has already been freed.
    ///
    /// # Examples
    ///
    /// Construct a `Motor`, free it, then construct a `RotationSensor` with the same Smart Port:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.free()
    /// # `motor` is now unusable, but port 1 can be used in another device, such as a `RotationSensor`
    /// rotation_sensor = RotationSensor(1)
    /// ```
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
