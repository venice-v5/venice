use std::cell::Cell;

use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{raise_stop_iteration, value_error},
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
    print::{Print, PrintKind},
};
use vex_sdk::{vexDeviceImuReset, vexDeviceImuStatusGet};
use vexide_devices::smart::{
    SmartDevice,
    imu::{InertialError, InertialOrientation, InertialSensor, InertialStatus},
};

use crate::{
    devices::{self},
    modvenice::{
        Exception, device_error, device_handle,
        math::{EulerZYX, Quaternion, Vec3},
        smart_port_index,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
        vasyncio::time32,
    },
    registry::SmartGuard,
};

/// An inertial sensor (IMU) plugged into a Smart Port.
///
/// This class provides an interface to interact with the V5 Inertial Sensor, which combines a
/// 3-axis accelerometer and 3-axis gyroscope for precise motion tracking and navigation
/// capabilities.
///
/// # Hardware Overview
///
/// The IMU's integrated accelerometer measures linear acceleration along three axes:
/// - X-axis: Forward/backward motion
/// - Y-axis: Side-to-side motion
/// - Z-axis: Vertical motion
///
/// These accelerometer readings include the effect of gravity, which can be useful for determining
/// the sensor's orientation relative to the ground.
///
/// The IMU also has a gyroscope that measures rotational velocity and position on three axes:
/// - Roll: Rotation around X-axis
/// - Pitch: Rotation around Y-axis
/// - Yaw: Rotation around Z-axis
///
/// Like all other Smart devices, VEXos will process sensor updates every 10mS.
///
/// # Coordinate System
///
/// The IMU uses a NED (North-East-Down) right-handed coordinate system:
/// - X-axis: Positive towards the front of the robot (North)
/// - Y-axis: Positive towards the right of the robot (East)
/// - Z-axis: Positive downwards (towards the ground)
///
/// This NED convention means that when the robot is on a flat surface:
/// - The Z acceleration will read approximately +9.81 m/s² (gravity)
/// - Positive roll represents clockwise rotation around the X-axis (when looking North)
/// - Positive pitch represents nose-down rotation around the Y-axis
/// - Positive yaw represents clockwise rotation around the Z-axis (when viewed from above)
///
/// # Calibration & Mounting Considerations
///
/// The IMU requires a calibration period to establish its reference frame in one of six possible
/// orientations (described by `InertialOrientation`). The sensor must be mounted flat in one of
/// these orientations. Readings will be unpredictable if the IMU is mounted at an angle or was
/// moving/disturbed during calibration.
///
/// In addition, physical pressure on the sensor's housing or static electricity can cause issues
/// with the onboard gyroscope, so pressure-mounting the IMU or placing the IMU low to the ground is
/// undesirable.
///
/// # Disconnect Behavior
///
/// If the IMU loses power due to a disconnect — even momentarily, all calibration data will be lost
/// and VEXos will re-initiate calibration automatically. The robot cannot be moving when this
/// occurs due to the aforementioned unpredictable behavior. As such, it is vital that the IMU
/// maintain a stable connection to the Brain and voltage supply during operation.
#[class(qstr!(InertialSensor))]
#[repr(C)]
pub struct InertialSensorObj {
    base: ObjBase,
    guard: SmartGuard<InertialSensor>,
}

/// Future that calibrates an IMU, created with `InertialSensor.Calibrate`.
#[class(qstr!(CalibrateFuture))]
#[repr(C)]
pub struct CalibrateFuture {
    base: ObjBase,
    state: Cell<CalibrateFutureState>,
    imu: Obj,
}

/// Represents one of six possible physical IMU orientations relative
/// to the earth's center of gravity.
#[class(qstr!(InertialOrientation))]
#[repr(C)]
pub struct InertialOrientationObj {
    base: ObjBase,
    orientation: InertialOrientation,
}

impl From<InertialError> for Exception {
    fn from(value: InertialError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl InertialOrientationObj {
    const fn new(orientation: InertialOrientation) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            orientation,
        }
    }

    /// X-axis facing down.
    #[constant]
    pub const X_DOWN: &Self = &Self::new(InertialOrientation::XDown);

    /// X-axis facing up.
    #[constant]
    pub const X_UP: &Self = &Self::new(InertialOrientation::XUp);

    /// Y-axis facing down.
    #[constant]
    pub const Y_DOWN: &Self = &Self::new(InertialOrientation::YDown);

    /// Y-axis facing up.
    #[constant]
    pub const Y_UP: &Self = &Self::new(InertialOrientation::YUp);

    /// Z-Axis facing up (VEX logo facing DOWN).
    #[constant]
    pub const Z_DOWN: &Self = &Self::new(InertialOrientation::ZDown);

    /// Z-Axis facing down (VEX logo facing UP).
    #[constant]
    pub const Z_UP: &Self = &Self::new(InertialOrientation::ZUp);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.orientation {
            InertialOrientation::XDown => "InertialOrientation.X_DOWN",
            InertialOrientation::XUp => "InertialOrientation.X_UP",
            InertialOrientation::YDown => "InertialOrientation.Y_DOWN",
            InertialOrientation::YUp => "InertialOrientation.Y_UP",
            InertialOrientation::ZDown => "InertialOrientation.Z_DOWN",
            InertialOrientation::ZUp => "InertialOrientation.Z_UP",
        })
    }
}

#[class_methods]
impl InertialSensorObj {
    /// The maximum time that the Inertial Sensor should take to *begin* its calibration process
    /// following a call to `InertialSensor.calibrate`. Measured in milliseconds.
    #[constant]
    const CALIBRATION_START_TIMEOUT_MS: i32 =
        InertialSensor::CALIBRATION_START_TIMEOUT.as_millis() as i32;

    /// The maximum time that the Inertial Sensor should take to *end* its calibration process after
    /// calibration has begun. Measured in milliseconds.
    #[constant]
    const CALIBRATION_END_TIMEOUT_MS: i32 =
        InertialSensor::CALIBRATION_END_TIMEOUT.as_millis() as i32;

    /// The minimum data rate that you can set an IMU to run at, in milliseconds.
    #[constant]
    pub const MIN_DATA_INTERVAL: i32 = InertialSensor::MIN_DATA_INTERVAL.as_millis() as i32;

    /// Create a new inertial sensor.
    ///
    /// # Important
    ///
    /// <section class="warning">
    ///
    /// This sensor must be calibrated using `InertialSensor.calibrate` before any meaningful
    /// data can be read from it.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
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
        let port = reader.next_positional()?;

        let guard = devices::lock_port(port, InertialSensor::new);

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    /// Calibrates the IMU.
    ///
    /// Returns a `CalibrateFuture` that resolves once the calibration operation has finished or
    /// timed out.
    ///
    /// This method MUST be called for any meaningful gyroscope readings to be obtained. Calibration
    /// requires the sensor to be sitting completely still. If the sensor is moving during the
    /// calibration process, readings will drift from reality over time.
    ///
    /// # Raises
    ///
    /// Calibration has a 1-second start timeout (when waiting for calibration to
    /// actually start on the sensor) and a 3-second end timeout (when waiting for calibration to
    /// complete after it has started) as a failsafe in the event that something goes wrong and the
    /// sensor gets stuck in a calibrating state.
    ///
    /// - `DeviceError`:  If either timeout is exceeded in its respective phase of calibration, or
    /// if there is not an Inertial Sensor connect to the port.
    ///
    /// # Examples
    ///
    /// Calibration process with error handling and a retry:
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     try:
    ///         imu.calibrate()
    ///         print("IMU calibrated successfully.")
    ///     except DeviceError as err:
    ///         print(f"IMU failed to calibrate, retrying. Reason: {err}")
    ///
    ///         # Since calibration failed, let's try one more time. If that fails,
    ///         # we just ignore the error and go on with our lives.
    ///         await imu.calibrate()
    ///
    /// vasyncio.run(main)
    /// ```
    ///
    /// Calibrating in a competition environment:
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// comp = Competition()
    ///
    /// @comp.autonomous
    /// async def auton():
    ///     while True:
    ///         try:
    ///             heading = imu.get_heading(DEGREES)
    ///             print(f"IMU Heading: {heading}°")
    ///         except DeviceError:
    ///             pass
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// async def main():
    ///     try:
    ///         await imu.calibrate()
    ///     except DeviceError:
    ///         # Log out a warning to terminal if calibration failed. You can also retry by
    ///         # calling it again, although this usually only happens if the sensor was unplugged.
    ///         println!("WARNING: IMU failed to calibrate! Readings might be inaccurate!");
    ///
    ///     await comp.run()
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    #[stub(sig = "(self) -> CalibrateFuture")]
    fn calibrate(self_in: Obj) -> CalibrateFuture {
        CalibrateFuture {
            base: ObjBase::new(CalibrateFuture::OBJ_TYPE),
            imu: self_in,
            state: Cell::new(CalibrateFutureState::Calibrate),
        }
    }

    /// Returns the Inertial Sensor’s yaw angle bounded from [0.0, 360.0) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise
    /// rotations are represented with negative ones.
    ///
    /// # Exceptions
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     try:
    ///         heading = imu.get_heading(DEGREES)
    ///         print(f"Heading is {heading} degrees.")
    ///     except DeviceError:
    ///         pass
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_heading(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().heading()?))
    }

    /// Sets the current reading of the sensor's heading to a given value.
    ///
    /// This only affects the value returned by `InertialSensor.get_heading` and does not effect
    /// `InertialSensor.get_rotation` or `InertialSensor.get_euler`/
    /// `InertialSensor.get_quaternion`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// # Set heading to 90 degrees clockwise.
    /// imu.set_heading(90, DEGREES)
    /// ```
    #[method]
    fn set_heading(&self, heading: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(heading);
        Ok(self.guard.borrow_mut().set_heading(angle)?)
    }

    /// Resets the current reading of the sensor's heading to zero.
    ///
    /// This only affects the value returned by `InertialSensor.get_heading` and does not effect
    /// `InertialSensor.get_rotation` or `InertialSensor.get_euler`/
    /// `InertialSensor.get_quaternion`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     # Store heading before reset.
    ///     heading = sensor.get_heading()
    ///
    ///     # Reset heading back to zero.
    ///     sensor.reset_heading()
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn reset_heading(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_heading()?)
    }

    /// Returns the total number of degrees the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive
    /// degree values, while counterclockwise rotations are represented with negative ones.
    ///
    /// # Errors
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     try:
    ///         rotation = imu.get_rotation(DEGREES)
    ///         print(f"Robot has rotated {rotation} degrees since calibration.")
    ///     except DeviceError:
    ///         pass
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_rotation(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().rotation()?))
    }

    /// Sets the current reading of the sensor's rotation to a given value.
    ///
    /// This only affects the value returned by `InertialSensor.get_rotation` and does not effect
    /// `InertialSensor.get_heading` or `InertialSensor.get_euler`/`InertialSensor.get_quaternion`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    /// imu.set_rotation(90, DEGREES)
    /// ```
    #[method]
    fn set_rotation(&self, rotation: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(rotation);
        Ok(self.guard.borrow_mut().set_rotation(angle)?)
    }

    /// Resets the current reading of the sensor's rotation to zero.
    ///
    /// This only affects the value returned by `InertialSensor.get_rotation` and does not effect
    /// `InertialSensor.get_heading` or `InertialSensor.get_euler`/`InertialSensor.get_quaternion`.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     # Store rotation before reset.
    ///     rotation = sensor.get_rotation(DEGREES)
    ///
    ///     # Reset heading back to zero.
    ///     sensor.reset_rotation()
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn reset_rotation(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_rotation()?)
    }

    /// Returns the Euler angles (pitch, yaw, roll) in radians representing the Inertial Sensor’s
    /// orientation.
    ///
    /// Euler angles are normalized to half a turn, meaning they range from (-180°, 180°].
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     euler = sensor.get_euler(DEGREES)
    ///     print("pitch: {euler.pitch}°, yaw: {euler.yaw}°, roll: {euler.roll}°")
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_euler(&self, unit: &RotationUnitObj) -> Result<EulerZYX, Exception> {
        Ok(EulerZYX::new(self.guard.borrow().euler()?, unit.unit()))
    }

    /// Returns a quaternion representing the Inertial Sensor’s current orientation.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     quaternion = sensor.get_quaternion()
    ///     print("x: {quaternion.x}, y: {quaternion.y}, z: {quaternion.z}, w: {quaternion.w}")
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_quaternion(&self) -> Result<Quaternion, Exception> {
        Ok(Quaternion::new(self.guard.borrow().quaternion()?))
    }

    /// Returns the Inertial Sensor’s raw gyroscope readings in dps (degrees per second).
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there is not an Inertial Sensor connected to the port, or if the sensor
    /// is currently calibrating and cannot be used.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Read out angular velocity values every 10mS
    ///     while True:
    ///         rates = sensor.get_gyro_rate()
    ///         print(f"x: {rates.x}°/s, y: {rates.y}°/s, z: {rates.z}°/s")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_gyro_rate(&self) -> Result<Vec3, Exception> {
        Ok(Vec3::new(self.guard.borrow().gyro_rate()?))
    }

    /// Returns the sensor's raw acceleration readings in g (multiples of ~9.8 m/s/s).
    ///
    /// # Errors
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     # Calibrate sensor, raise if calibration fails.
    ///     await imu.calibrate()
    ///
    ///     # Read out angular velocity values every 10mS
    ///     while True:
    ///         acceleration = sensor.get_acceleration()
    ///         print(f"x: {acceleration.x}G, y: {acceleration.y}G, z: {acceleration.z}G")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_acceleration(&self) -> Result<Vec3, Exception> {
        Ok(Vec3::new(self.guard.borrow().acceleration()?))
    }

    /// Returns the internal status code of the inertial sensor.
    ///
    /// # Errors
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    /// status = imu.get_status()
    ///
    /// print(f"Status: {status:b}")
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()?.bits() as i32)
    }

    /// Returns `True` if the sensor is currently calibrating.
    ///
    /// # Errors
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// # We haven't calibrated yet, so this is expected.
    /// if !imu.is_calibrating():
    ///     print("Sensor is not currently calibrating.")
    /// ```
    #[method]
    fn is_calibrating(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_calibrating()?)
    }

    /// Returns `True` if the sensor was calibrated using auto-calibration.
    ///
    /// In some cases (such as a loss of power), VEXos will automatically decide to recalibrate the
    /// inertial sensor.
    ///
    /// # Errors
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InterialSensor(1)
    ///
    /// if sensor.is_auto_calibrated():
    ///     print("Sensor was automatically calibrated by VEXos.")
    /// ```
    #[method]
    fn is_auto_calibrated(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_auto_calibrated()?)
    }

    /// Returns the physical orientation of the sensor measured at calibration.
    ///
    /// This orientation can be one of six possible orientations aligned to two cardinal directions.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// async def main():
    ///     await imu.calibrate()
    ///     orientation = imu.get_physical_orientation()
    ///     print(f"Sensor was calibrated while facing {orientation}")
    ///
    /// vasyncio.run(main)
    #[method]
    #[stub(sig = "(self) -> InertialOrientation")]
    fn get_physical_orientation(&self) -> Result<Obj, Exception> {
        Ok(match self.guard.borrow().physical_orientation()? {
            InertialOrientation::XDown => Obj::from_static(InertialOrientationObj::X_DOWN),
            InertialOrientation::XUp => Obj::from_static(InertialOrientationObj::X_UP),

            InertialOrientation::YDown => Obj::from_static(InertialOrientationObj::Y_DOWN),
            InertialOrientation::YUp => Obj::from_static(InertialOrientationObj::Y_UP),

            InertialOrientation::ZDown => Obj::from_static(InertialOrientationObj::Z_DOWN),
            InertialOrientation::ZUp => Obj::from_static(InertialOrientationObj::Z_UP),
        })
    }

    /// Sets the internal computation speed of the IMU.
    ///
    /// This method does NOT change the rate at which user code can read data off the IMU, as the
    /// brain will only talk to the device every 10mS regardless of how fast data is being sent
    /// or computed.
    ///
    /// This duration should be above `InertialSensor.MIN_DATA_INTERVAL` (5 milliseconds).
    ///
    /// # Precision
    ///
    /// The internal data rate of the IMU has a precision of 5 milliseconds.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    /// - `ValueError`: If `interval` is negative.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// imu = InertialSensor(1)
    ///
    /// # Set to minimum interval.
    /// imu.set_data_interval(InertialSensor.MIN_DATA_INTERVAL)
    /// ```
    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        if interval < 0.0 {
            Err(value_error(c"interval cannot be negative"))?
        }
        let dur = unit.unit().float_to_dur(interval);
        self.guard.borrow_mut().set_data_interval(dur)?;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CalibrationPhase {
    Status,
    Start,
    End,
}

#[derive(Clone, Copy)]
pub enum CalibrateFutureState {
    Calibrate,
    Waiting {
        timestamp: time32::Instant,
        phase: CalibrationPhase,
    },
}

#[class_methods]
impl CalibrateFuture {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
        let this = self_in.try_as_obj::<Self>().unwrap();

        let imu = this
            .imu
            .try_as_obj::<InertialSensorObj>()
            .unwrap()
            .guard
            .borrow();

        let device = unsafe { device_handle(smart_port_index(imu.port_number())) };

        // Get the sensor's status flags, which tell us whether or not we are still calibrating.
        let status = InertialStatus::from_bits_retain(if let Err(e) = imu.validate_port() {
            // IMU got unplugged, so we'll resolve early.
            Exception::from(e).raise(token());
        } else {
            // Get status flags from VEXos.
            let flags = unsafe { vexDeviceImuStatusGet(device) };
            if flags == 0x0 {
                this.state.set(CalibrateFutureState::Waiting {
                    timestamp: time32::Instant::now(),
                    phase: CalibrationPhase::Status,
                });
            }

            flags
        });

        match this.state.get() {
            // The "calibrate" phase begins the calibration process.
            //
            // self only happens for one poll of the future (the first one). All future polls will
            // either be waiting for calibration to start or for calibration to end.
            CalibrateFutureState::Calibrate => {
                // Check if the sensor was already calibrating before we recalibrate it ourselves.
                //
                // self can happen at the start of program execution or if the sensor loses then
                // regains power. In those instances, VEXos will automatically start
                // the calibration process without us asking.
                // Calling [`vexDeviceImuReset`] while calibration is already happening has caused
                // bugs in our testing, so we instead just want to wait until the
                // calibration attempt has finished.
                //
                // See <https://github.com/vexide/vexide/issues/253> for more details.
                if status.contains(InertialStatus::CALIBRATING) {
                    // Sensor was already calibrating, so wait for that to finish.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::End,
                    });
                } else {
                    // Request that VEXos calibrate the IMU, and transition to pending state.
                    unsafe { vexDeviceImuReset(device) }

                    // Change to waiting for calibration to start.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::Start,
                    });
                }

                Obj::NONE
            }

            // In self stage, we are either waiting for the calibration status flag to be set
            // (CalibrationPhase::Start), indicating that calibration has begun, or we
            // are waiting for the calibration status flag to be cleared, indicating
            // that calibration has finished (CalibrationFlag::End).
            CalibrateFutureState::Waiting {
                timestamp: since,
                phase,
            } => {
                let elapsed = time32::Instant::now() - since;

                if elapsed
                    > time32::Duration::from_duration(match phase {
                        CalibrationPhase::Start | CalibrationPhase::Status => {
                            InertialSensor::CALIBRATION_START_TIMEOUT
                        }
                        CalibrationPhase::End => InertialSensor::CALIBRATION_END_TIMEOUT,
                    })
                {
                    // Waiting took too long and exceeded a timeout.
                    device_error(c"calibration timed out").raise(token());
                }

                if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start
                {
                    // We are in the "start" phase (waiting for the flag to be set) and the flag is
                    // now set, meaning that calibration has begun.
                    //
                    // We now know that the sensor is actually calibrating, so we transition to
                    // [`CalibrationPhase::End`] and reset the timeout timestamp to wait for
                    // calibration to finish.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::End,
                    });
                } else if !status.is_empty() && phase == CalibrationPhase::Status {
                    this.state.set(CalibrateFutureState::Calibrate);
                } else if !status.contains(InertialStatus::CALIBRATING)
                    && phase == CalibrationPhase::End
                {
                    // The [`InertialStatus::CALIBRATING`] has been cleared, indicating that
                    // calibration is complete.
                    raise_stop_iteration(token(), Obj::NONE);
                }

                Obj::NONE
            }
        }
    }
}
