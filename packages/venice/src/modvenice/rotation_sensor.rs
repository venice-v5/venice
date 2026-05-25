use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::{math::Direction, smart::rotation::RotationSensor};

use crate::{
    devices::{self},
    modvenice::{
        Exception,
        motor::direction::DirectionObj,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    registry::SmartGuard,
};

#[class(qstr!(RotationSensor))]
#[repr(C)]
pub struct RotationSensorObj {
    base: ObjBase,
    guard: SmartGuard<RotationSensor>,
}

/// A rotation sensor plugged into a Smart Port.
///
/// This class provides an interface to interact with the VEX V5 Rotation Sensor, which measures
/// the absolute position, rotation count, and angular velocity of a rotating shaft.
///
/// # Hardware Overview
///
/// The sensor provides absolute rotational position tracking from 0° to 360° with 0.088° accuracy.
/// The sensor is compromised of two magnets which utilize the [Hall Effect] to indicate angular
/// position. A chip inside the rotation sensor (a Cortex M0+) then keeps track of the total
/// rotations of the sensor to determine total position traveled.
///
/// Position is reported by VEXos in centidegrees before being converted to the requested rotation
/// type.
///
/// The absolute angle reading is preserved across power cycles (similar to a potentiometer), while
/// the position count stores the cumulative forward and reverse revolutions relative to program
/// start, however the *position* reading will be reset if the sensor loses power. Angular velocity
/// is measured in degrees per second.
///
/// Like all other Smart devices, VEXos will process sensor updates every 10mS.
///
/// [Hall Effect]: https://en.wikipedia.org/wiki/Hall_effect_sensor
#[class_methods]
impl RotationSensorObj {
    /// The minimum data rate that you can set a rotation sensor to, in milliseconds.
    #[constant]
    const MIN_DATA_INTERVAL_MS: i32 = RotationSensor::MIN_DATA_INTERVAL.as_millis() as i32;

    /// The amount of unique sensor readings per one revolution of the sensor.
    #[constant]
    const TICKS_PER_REVOLUTION: i32 = RotationSensor::TICKS_PER_REVOLUTION as i32;

    /// Creates a new rotation sensor on the given port.
    ///
    /// Whether or not the sensor should be reversed on creation can be specified.
    ///
    /// # Examples
    ///
    /// ```
    /// from venice import *
    ///
    /// sensor = RotationSensor(1)
    /// ```
    #[make_new]
    #[stub(sig = "(self, port: int, direction: Direction = Direction.FORWARD) -> None")]
    fn new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 2).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let direction = reader
            .next_positional_or(DirectionObj::FORWARD)?
            .direction();

        let guard = devices::lock_port(port, |port| RotationSensor::new(port, direction));

        Ok(RotationSensorObj {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let angle = self.guard.borrow_mut().angle()?;
        Ok(unit.unit().angle_to_float(angle))
    }

    /// Returns the total accumulated rotation of the sensor over time.
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
    /// sensor = RotationSensor(1)
    /// position = sensor.get_position(DEGREES)
    ///
    /// print(f"Position in degrees: {position}")
    /// ```
    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let position = self.guard.borrow_mut().position()?;
        Ok(unit.unit().angle_to_float(position))
    }

    /// Sets the sensor's position reading.
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
    /// sensor = RotationSensor(1)
    ///
    /// # Set position to 15 degrees
    /// sensor.set_position(15, DEGREES)
    /// ```
    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position);
        self.guard.borrow_mut().set_position(angle)?;
        Ok(())
    }

    /// Returns the sensor's current velocity in degrees per second.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = RotationSensor(1)
    ///
    /// velocity = sensor.get_velocity()
    /// print(f"Velocity in RPM: {velocity / 6}") # 1rpm = 6dps
    /// ```
    #[method]
    fn get_velocity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow_mut().velocity()? as f32)
    }

    /// Reset's the sensor's position reading to zero.
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
    /// sensor = RotationSensor(1)
    ///
    /// print(f"Before reset: {sensor.get_position()}")
    /// sensor.reset_position()
    /// print(f"After reset: {sensor.get_position()}")
    /// ```
    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        self.guard.borrow_mut().reset_position()?;
        Ok(())
    }

    /// Sets the sensor to operate in a given `Direction`.
    ///
    /// This determines which way the sensor considers to be “forwards”. You can use the marking on
    /// the top of the motor as a reference:
    ///
    /// - When `Direction.FORWARD` is specified, positive velocity/voltage values will cause the
    ///   motor to rotate **with the arrow on the top**. Position will increase as the motor rotates
    ///   **with the arrow**.
    /// - When `Direction.REVERSE` is specified, positive velocity/voltage values will cause the
    ///   motor to rotate **against the arrow on the top**. Position will increase as the motor
    ///   rotates **against the arrow**.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Set the sensor's direction to `Direction.Reverse`.
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = Rotation(1)
    ///
    /// # Reverse the sensor
    /// sensor.set_direction(Direction.REVERSE)
    /// ```
    ///
    /// Reverse the sensor's direction (set to opposite of the previous direction):
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = RotationSensor(1)
    ///
    /// # Reverse the sensor
    /// sensor.set_direction(~sensor.get_direction())
    /// ```
    #[method]
    fn set_direction(&self, direction: &DirectionObj) -> Result<(), Exception> {
        self.guard
            .borrow_mut()
            .set_direction(direction.direction())?;
        Ok(())
    }

    /// Returns the `Direction` of this sensor.
    ///
    /// # Examples
    ///
    /// ```
    /// from venice import *
    ///
    /// sensor = RotationSensor(1)
    /// if sensor.get_direction() == Direction.FORWARD:
    ///     print("Sensor's direction is forward")
    /// else:
    ///     print("Sensor's direction is reverse")
    /// ```
    // Venice's convention for deciding between using getters/setters and attributes is that
    // attributes should never perform SDK calls. If loading or storing some `x` requires calling
    // into the SDK, then that functionality should be moved into getters and setters `get_x` and
    // `set_x`.
    //
    // Loading rotation sensor direction does not require an SDK call, but storing it does
    // (`set_direction`). It's possible to define `direction` as an attribute and make it
    // read-only, but clash with the separate setter API and be misleading for users. That's why,
    // despite not requiring an SDK call, loading direction is still a getter method instead of an
    // attribute.
    #[method]
    #[stub(sig = "(self) -> Direction")]
    fn get_direction(&self) -> Obj {
        let dir = self.guard.borrow().direction();
        Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        })
    }

    /// Returns the sensor's internal status code.
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
    /// sensor = RotationSensor(1)
    /// status = sensor.get_status()
    /// print(f"Status: {status:b}")
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    /// Sets the internal computation speed of the rotation sensor.
    ///
    /// This method does NOT change the rate at which user code can read data off the sensor, as the
    /// brain will only talk to the device every 10mS regardless of how fast data is being sent or
    /// computed.
    ///
    /// This duration should be above `Self.MIN_DATA_INTERVAL` (5 milliseconds).
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
    /// sensor = RotationSensor(1)
    ///
    /// # Set to minimum interval.
    /// sensor.set_data_interval(RotationSensor.MIN_DATA_INTERVAL)
    /// ```
    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        self.guard
            .borrow_mut()
            .set_data_interval(unit.unit().float_to_dur(interval))?;
        Ok(())
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
    /// Construct a `RotationSensor`, free it, then construct a `Motor` with the same Smart Port:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = RotationSensor(1)
    /// sensor.free()
    /// # `sensor` is now unusable, but port 1 can be used in another device, such as a `Motor`
    /// motor = Motor(1)
    /// ```
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
