use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::gps::GpsSensor;

use crate::{
    devices,
    modvenice::{
        Exception,
        math::{EulerZYX, Point2, Quaternion, Vec3},
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
    },
    registry::SmartGuard,
};

/// A GPS sensor plugged into a Smart Port.
///
/// This module provides an interface to interact with the VEX V5 Game Position System (GPS) Sensor,
/// which uses computer vision and an inertial measurement unit (IMU) to provide absolute position
/// tracking within a VEX Robotics Competition field.
///
/// # Hardware Description
///
/// The GPS sensor combines a monochrome camera and an IMU for robust position tracking through
/// visual odometry. It works by detecting QR-like patterns on the field perimeter, using both the
/// pattern sequence's and apparent size for position determination. The integrated IMU provides
/// motion tracking for position estimation when visual tracking is unavailable or unreliable.
///
/// The sensor has specific operating ranges: it requires a minimum distance of 20 inches from the
/// field perimeter for reliable readings, has a deadzone between 0-13.5 inches, and maintains
/// accuracy up to 12 feet from the perimeter.
///
/// Sensor fusion between the camera and IMU helps maintain position tracking through dead zones and
/// areas of inconsistent visual detection.
///
/// Further information about the sensor's method of operation can be found in [IFI's patent](https://docs.google.com/viewerng/viewer?url=https://patentimages.storage.googleapis.com/4f/74/30/eccf334da0ae38/WO2020219788A1.pdf).
#[class(qstr!(GpsSensor))]
#[repr(C)]
pub struct GpsSensorObj {
    base: ObjBase,
    guard: SmartGuard<GpsSensor>,
}

#[class_methods]
impl GpsSensorObj {
    /// Creates a new GPS sensor.
    ///
    /// # Sensor Configuration
    ///
    /// The sensor requires three measurements to be made at the start of a match, passed as
    /// arguments to this function:
    ///
    /// ## Sensor Offset
    ///
    /// `offset` is the physical offset of the sensor's mounting location from a reference point on
    /// the robot.
    ///
    /// Offset defines the exact point on the robot that is considered a "source of truth" for the
    /// robot's position. For example, if you considered the center of your robot to be the
    /// reference point for coordinates, then this value would be the signed 4-quadrant x and y
    /// offset from that point on your robot in meters. Similarly, if you considered the sensor
    /// itself to be the robot's origin of tracking, then this value would simply be `Point2(0, 0)`
    ///
    /// ## Initial Robot Position
    ///
    /// `initial_position` is an estimate of the robot's initial cartesian coordinates on the field
    /// in meters. This value helpful for cases when the robot's starting point is near a field
    /// wall.
    ///
    /// When the GPS Sensor is too close to a field wall to properly read the GPS strips, the sensor
    /// will be unable to localize the robot's position due the wall's proximity limiting the view
    /// of the camera. This can cause the sensor inaccurate results at the start of a match, where
    /// robots often start directly near a wall.
    ///
    /// By providing an estimate of the robot's initial position on the field, this problem is
    /// partially mitigated by giving the sensor an initial frame of reference to use.
    ///
    /// # Initial Robot Heading
    ///
    /// `initial_heading` is a value between 0 and 360 degrees that informs the GPS of its heading
    /// at the start of the match. Similar to `initial_position`, this is useful for improving
    /// accuracy when the sensor is in close proximity to a field wall, as the sensor's rotation
    /// values are continuously checked against the GPS field strips to prevent drift over time. If
    /// the sensor starts too close to a field wall, providing an `initial_heading` can help prevent
    /// this drift at the start of the match.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a GPS sensor mounted 22.5 cm left and 22.5 cm forward of the robot's tracking origin.
    /// # Starting at position (0, 0) with a 90 degree heading.
    /// gps = GpsSensor(
    ///     # Port 1
    ///     1,
    ///
    ///     # Sensor is mounted 0.225 meters to the left and 0.225 meters forward from the robot's tracking origin.
    ///     Point2(-0.225, 0.225),
    ///
    ///     # Robot's starting point is at the center of the field.
    ///     Point2(0, 0),
    ///
    ///     # Robot is facing to the right initially.
    ///     90,
    ///     DEGREES
    /// )
    /// ```
    #[make_new]
    #[stub(
        sig = "(self, port: int, offset: Point2, initial_position: Point2, initial_heading: float, initial_heading_unit: RotationUnit, /) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();

        let port_number = reader.next_positional()?;
        let offset = reader.next_positional::<&Point2>()?;
        let initial_position = reader.next_positional::<&Point2>()?;
        let initial_heading = reader.next_positional::<f32>()?;
        let initial_heading_unit = reader.next_positional::<&RotationUnitObj>()?;

        let initial_heading_angle = initial_heading_unit.unit().float_to_angle(initial_heading);

        Ok(Self {
            guard: devices::lock_port(port_number, |port| {
                GpsSensor::new(
                    port,
                    offset.as_vexide_point2(),
                    initial_position.as_vexide_point2(),
                    initial_heading_angle.as_degrees(),
                )
            }),
            base: ty.into(),
        })
    }

    /// Returns the user-configured offset from a reference point on the robot.
    ///
    /// This offset value is passed to `GpsSensor()` and can be changed using
    /// `GpsSensor.set_offset`.
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
    /// gps = GpsSensor(
    ///     1,
    ///
    ///     # Initial offset value is configured here!
    ///     #
    ///     # Let's assume that the sensor is mounted 22.5 cm to the left and 22.5 cm forward from
    ///     # our desired tracking origin
    ///     Point2(-0.225, 0.225),
    ///
    ///     Point2(0, 0),
    ///     90,
    ///     DEGREES
    /// )
    ///
    /// # Get the configured offset of the sensor
    /// offset = gps.get_offset()
    /// print(f"GPS sensor is mounted at x={offset.x}, y={offset.y}") # "Sensor is mounted at x=-0.225, y=0.225"
    ///
    /// # Change the offset to something new
    /// gps.set_offset(Point2(0, 0))
    ///
    /// # Get the configured offset of the sensor again
    /// new_offset = gps.get_offset()
    /// print(f"GPS sensor is mounted at x={new_offset.x}, y={new_offset.y}") # "Sensor is mounted at x=0.0, y=0.0"
    /// ```
    #[method]
    fn get_offset(&self) -> Result<Point2, Exception> {
        Ok(self.guard.borrow().offset()?.into())
    }

    /// Adjusts the sensor's physical offset from the robot's tracking origin.
    ///
    /// This value is also configured initially through `GpsSensor::()`.
    ///
    /// Offset defines the exact point on the robot that is considered a "source of truth" for the
    /// robot's position. For example, if you considered the center of your robot to be the
    /// reference point for coordinates, then this value would be the signed 4-quadrant x and y
    /// offset from that point on your robot in meters. Similarly, if you considered the sensor
    /// itself to be the robot's origin of tracking, then this value would simply be `Point2(0, 0)`
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
    /// gps = GpsSensor(
    ///     1,
    ///
    ///     # Initial offset value is configured here!
    ///     #
    ///     # Let's assume that the sensor is mounted 22.5 cm to the left and 22.5 cm forward from
    ///     # our desired tracking origin
    ///     Point2(-0.225, 0.225),
    ///
    ///     Point2(0, 0),
    ///     90,
    ///     DEGREES
    /// )
    ///
    /// # Get the configured offset of the sensor
    /// offset = gps.get_offset()
    /// print(f"GPS sensor is mounted at x={offset.x}, y={offset.y}") # "Sensor is mounted at x=-0.225, y=0.225"
    ///
    /// # Change the offset to something new
    /// gps.set_offset(Point2(0, 0))
    ///
    /// # Get the configured offset of the sensor again
    /// new_offset = gps.get_offset()
    /// print(f"GPS sensor is mounted at x={new_offset.x}, y={new_offset.y}") # "Sensor is mounted at x=0.0, y=0.0"
    /// ```
    #[method]
    fn set_offset(&self, offset: &Point2) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_offset(offset.as_vexide_point2())?)
    }

    /// Returns an estimate of the robot's location on the field as cartesian coordinates measured
    /// in meters.
    ///
    /// The reference point for a robot's position is determined by the sensor's configured offset
    /// value.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Get current position
    /// position = gps.get_position()
    /// print(f"Robot is at x={position.x}, y={position.y}")
    /// ```
    #[method]
    fn get_position(&self) -> Result<Point2, Exception> {
        Ok(self.guard.borrow().position()?.into())
    }

    /// Returns the sensor's yaw angle bounded by [0.0, 360.0) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise
    /// rotations are represented with negative ones. If a heading offset has not been set using
    /// `GpsSensor.set_heading`, then 90 degrees will located to the right of the field.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Get current heading
    /// heading = gps.get_heading(DEGREES)
    /// print(f"Heading is {heading} degrees")
    /// ```
    #[method]
    fn get_heading(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().heading()?))
    }

    /// Offsets the reading of `GpsSensor.get_heading` to a specified angle value.
    ///
    /// Target will default to `360.0` if above `360.0` and default to `0.0` if below `0.0`.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Set heading to 90 degrees clockwise.
    /// gps.set_heading(90, DEGREES)
    ///
    /// heading = gps.get_heading(DEGREES)
    /// print(f"Heading: {heading} degrees")
    /// ```
    #[method]
    fn set_heading(&self, heading: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_heading(unit.unit().float_to_angle(heading))?)
    }

    /// Offsets the reading of `GpsSensor.get_heading` to zero.
    ///
    /// This method has no effect on the values returned by `GpsSensor.get_position`.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// async def main():
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     # Store heading before reset.
    ///     heading = gps.get_heading(DEGREES)
    ///
    ///     # Reset heading back to zero.
    ///     gps.reset_heading()
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn reset_heading(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_heading()?)
    }

    /// Returns the total number of degrees the GPS has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive
    /// degree values, while counterclockwise rotations are represented with negative ones. If a
    /// heading offset has not been set using `GpsSensor.set_rotation`, then 90 degrees will
    /// located to the right of the field.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// rotation = gps.get_rotation(DEGREES)
    /// print(f"Robot has rotated {rotation} degrees since calibration.")
    /// ```
    #[method]
    fn get_rotation(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.guard.borrow().rotation()?))
    }

    /// Offsets the reading of `GpsSensor.get_rotation` to a specified angle value.
    ///
    /// This method has no effect on the values returned by `GpsSensor.get_position`.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Set rotation to 90 degrees clockwise.
    /// gps.set_rotation(90, DEGREES)
    ///
    /// rotation = gps.get_rotation(DEGREES)
    /// print(f"Rotation: {rotation} degrees")
    /// ```
    #[method]
    fn set_rotation(&self, rotation: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_rotation(unit.unit().float_to_angle(rotation))?)
    }

    /// Offsets the reading of `GpsSensor.get_rotation` to zero.
    ///
    /// This method has no effect on the values returned by `GpsSensor.get_position`.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// async def main():
    ///     # Sleep for two seconds to allow the robot to be moved.
    ///     await vasyncio.Sleep(2, SECOND)
    ///
    ///     # Store rotation before reset.
    ///     rotation = gps.get_rotation(DEGREES)
    ///
    ///     # Reset rotation back to zero.
    ///     gps.reset_rotation()
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn reset_rotation(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_rotation()?)
    }

    /// Returns the Euler angles (pitch, yaw, roll) representing the GPS's orientation.
    ///
    /// Euler angles are normalized to half a turn, meaning they range from (-180°, 180°].
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// angles = gps.get_euler(DEGREES)
    /// print("yaw: {angles.yaw}°, pitch: {angles.pitch}°, roll: {angles.roll}°")
    /// ```
    #[method]
    fn get_euler(&self, unit: &RotationUnitObj) -> Result<EulerZYX, Exception> {
        Ok(EulerZYX::new(self.guard.borrow().euler()?, unit.unit()))
    }

    /// Returns a quaternion representing the sensor's orientation.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// quaternion = gps.get_quaternion()
    /// print(f"x: {quaternion.x}, y: {quaternion.y}, z: {quaternion.z}, scalar: {quaternion.w}")
    /// ```
    #[method]
    fn get_quaternion(&self) -> Result<Quaternion, Exception> {
        Ok(Quaternion::new(self.guard.borrow().quaternion()?))
    }

    /// Returns raw accelerometer values of the sensor's internal IMU.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// async def main():
    ///     # Read out acceleration values every 10 ms
    ///     while True:
    ///         acceleration = gps.get_acceleration()
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

    /// Returns the raw gyroscope values of the sensor's internal IMU.
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
    /// # Assume we're starting in the middle of the field facing upwards, with the sensor's
    /// # mounting point being our reference for position.
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// async def main():
    ///     # Read out angular velocity values every 10 ms
    ///     while True:
    ///         rates = gps.get_gyro_rate()
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

    /// Returns the RMS (Root Mean Squared) error for the sensor's position reading in meters.
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
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Check position accuracy
    /// error = gps.get_error()
    /// if error > 0.3:
    ///     print(f"Warning: GPS position accuracy is low ({error} m)")
    /// ```
    #[method]
    fn get_error(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().error()? as f32)
    }

    /// Returns the internal status code of the sensor.
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
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// status = gps.get_status()
    /// print(f"Status: 0x{status:08x}")
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        // cast from u32 to i32, should be OK since the amount of bits is preserved and no data is
        // lost
        Ok(self.guard.borrow().status()? as i32)
    }

    /// Sets the internal computation speed of the sensor's internal IMU.
    ///
    /// This method does NOT change the rate at which user code can read data off the GPS, as the
    /// brain will only talk to the device every 10mS regardless of how fast data is being sent or
    /// computed. This also has no effect on the speed of methods such as `GpsSensor.get_position`,
    /// as it only changes the *internal* computation speed of the sensor's internal IMU.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    /// - `ValueError`: If `interval` is negative, non-finite, or too large to represent.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// gps = GpsSensor(
    ///     1,
    ///     Point2(0, 0),
    ///     Point2(0, 0),
    ///     0,
    ///     DEGREES
    /// )
    ///
    /// # Set to minimum interval
    /// gps.set_data_interval(5, MILLIS)
    /// ```
    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) -> Result<(), Exception> {
        let duration = unit.unit().float_to_dur(interval)?;
        Ok(self.guard.borrow_mut().set_data_interval(duration)?)
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
    /// print(f"Status: 0x{status:08x}")
    /// ```
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
