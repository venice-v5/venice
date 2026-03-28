"""
Venice is an open-source Micropython runtime for VEX V5 robots.

```python
from venice import *

async def main():
    my_motor = Motor(
        1,
        Direction.FORWARD,
        Gearset.GREEN
    )
    my_motor.set_voltage(10.0)

    while True:
        await vasyncio.Sleep(10, TimeUnit.MILLIS)

vasyncio.run(main())
```
"""

from __future__ import annotations
from . import vasyncio as vasyncio
from typing import ClassVar, Never

class BrakeMode:
    """Determines the behavior a motor should use when braking with `Motor.brake`.

    Smart motors support three braking behaviors. `BrakeMode.COAST` lets the motor spin down
    freely, `BrakeMode.BRAKE` uses regenerative braking to slow the motor more quickly, and
    `BrakeMode.HOLD` actively applies force to maintain the motor's current position.
    """

    COAST: ClassVar['BrakeMode']
    """Motor never brakes."""

    BRAKE: ClassVar['BrakeMode']
    """Motor uses regenerative braking to slow down faster."""

    HOLD: ClassVar['BrakeMode']
    """Motor exerts force holding itself in the same position."""


BrakeMode.COAST = BrakeMode()
BrakeMode.BRAKE = BrakeMode()
BrakeMode.HOLD = BrakeMode()


class Direction:
    """A rotational direction.

    This enum describes whether something should rotate in its forward or reverse direction.
    """

    FORWARD: ClassVar['Direction']
    """Rotates in the forward direction."""

    REVERSE: ClassVar['Direction']
    """Rotates in the reverse direction."""


Direction.FORWARD = Direction()
Direction.REVERSE = Direction()


class Gearset:
    """Internal gearset used by VEX Smart motors.

    Smart motors support three gearsets, commonly identified by cartridge color.
    """

    RED: ClassVar['Gearset']
    """36:1 gear ratio."""

    GREEN: ClassVar['Gearset']
    """18:1 gear ratio."""

    BLUE: ClassVar['Gearset']
    """6:1 gear ratio."""


Gearset.RED = Gearset()
Gearset.GREEN = Gearset()
Gearset.BLUE = Gearset()


class RotationUnit:
    """A unit used to represent rotational measurements."""

    RADIANS: ClassVar['RotationUnit']
    """Rotation measured in radians."""

    DEGREES: ClassVar['RotationUnit']
    """Rotation measured in degrees."""

    TURNS: ClassVar['RotationUnit']
    """Rotation measured in full turns."""


RotationUnit.RADIANS = RotationUnit()
RotationUnit.DEGREES = RotationUnit()
RotationUnit.TURNS = RotationUnit()


class MotorType:
    """Represents the type of a Smart motor.

    Either a 11W (V5) or 5.5W (EXP) motor.
    """

    EXP: ClassVar['MotorType']
    """A 5.5W Smart Motor."""

    V5: ClassVar['MotorType']
    """An 11W Smart Motor."""


MotorType.EXP = MotorType()
MotorType.V5 = MotorType()


class Motor:
    """A motor plugged into a Smart Port.

    This class provides abstractions for interacting with VEX Smart Motors, supporting both the
    11W and 5.5W variants.

    ## Overview

    The V5 Smart Motors come in two variants: [an 11W model](https://www.vexrobotics.com/276-4840.html),
    with interchangeable gear cartridges and [a 5.5W model](https://www.vexrobotics.com/276-4842.html),
    with a fixed gearing. The 11W motor supports three cartridge options, which will gear the motor
    down from its base RPM of 3600: a red cartridge providing 100 RPM output, a green cartridge for
    200 RPM, and a blue cartridge for 600 RPM. The 5.5W motor comes with a non-interchangeable 200
    RPM gear cartridge.

    Smart Motors feature several integrated sensors, including an encoder for measuring the velocity
    and position of the motor, a temperature sensor for detecting overheats, and sensors for
    measuring output voltage, current, and efficiency.

    Communication between a Smart motor and the V5 Brain occur at two different intervals. While
    the motor communicates with the Brain every 5 milliseconds (and commands can be written to the
    motor every 5mS), the Brain only reads data from the motor every 10mS. This effectively places
    the data *write* interval at 5mS and the data *read* interval at 10mS.

    More in-depth specs for the 11W motor can be found [here](https://kb.vex.com/hc/en-us/articles/360060929971-Understanding-V5-Smart-Motors).

    ## Current Limitations

    There are some cases where VEXos or the motor itself may decide to limit output current:

    - **Stall Prevention**: The stall current on 11W motors is limited to 2.5A. This limitation
      eliminates the need for automatic resetting fuses (PTC devices) in the motor, which can
      disrupt operation. By restricting the stall current to 2.5A, the motor effectively avoids
      undesirable performance dips and ensures that users do not inadvertently cause stall
      situations.
    - **Motor Count**: Robots that use 8 or fewer 11W motors will have the aforementioned current
      limit of 2.5A set for each motor. Robots using more than 8 motors will have a lower default
      current limit per motor than 2.5A. This limit is determined in VEXos by a calculation
      accounting for the number of motors plugged in, and the user's manually set current limits
      using `Motor.set_current_limit`. For more information regarding the current limiting behavior
      of VEXos, see [this forum post](https://www.vexforum.com/t/how-does-the-decreased-current-affect-the-robot-when-using-more-than-8-motors/72650/4).
    - **Temperature Management**: Motors have an onboard sensor for measuring internal temperature.
      If the motor determines that it is overheating, it will throttle its output current and warn
      the user.

    ## Motor Control

    Each motor contains a sophisticated control system built around a Cortex M0+ microcontroller.
    The microcontroller continuously monitors position, speed, direction, voltage, current, and
    temperature through integrated sensors.

    The onboard motor firmware implements a set of pre-tuned PID (Proportional-Integral-Derivative)
    controllers operating on a 10-millisecond cycle for position and velocity control. Motors also
    have braking functionality for holding a specific position under load.
    """

    def __init__(
        self,
        port: int,
        direction: Direction = Direction.FORWARD,
        gearset: Gearset = Gearset.GREEN,
    ) -> None:
        """Create a new 11W (V5) Smart Motor.

        `port` must be between 1 and 21.
        """
        ...

    @classmethod
    def new_exp(
        cls,
        port: int,
        direction: Direction = Direction.FORWARD,
    ) -> 'Motor':
        """Create a new 5.5W (EXP) Smart Motor.

        `port` must be between 1 and 21.
        """
        ...

    def set_voltage(self, volts: float) -> None:
        """Set the motor's output voltage.

        This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards)
        volts, and controls the raw output of the motor.
        """
        ...

    def set_velocity(self, rpm: int) -> None:
        """Spin the motor at a target velocity.

        This velocity corresponds to different actual speeds in RPM depending on the gearset used
        for the motor. Velocity is held with an internal PID controller to ensure consistent
        speed, as opposed to setting the motor's voltage.
        """
        ...

    def brake(self, mode: BrakeMode) -> None:
        """Stop this motor with the given `BrakeMode`."""
        ...

    def set_gearset(self, gearset: Gearset) -> None:
        """Set the gearset of an 11W motor.

        EXP motors have no swappable gearset.
        """
        ...

    is_exp: bool
    """Whether this motor is a 5.5W (EXP) Smart Motor."""

    is_v5: bool
    """Whether this motor is an 11W (V5) Smart Motor."""

    max_voltage: float
    """The maximum voltage for this motor based on its motor type."""

    def get_gearset(self) -> Gearset:
        """Return the gearset of the motor.

        For 5.5W motors, this is always returned as `Gearset.GREEN`.
        """
        ...

    def set_position_target(self, position: float, unit: RotationUnit, velocity: int) -> None:
        """Set an absolute position target for the motor to attempt to reach.

        `position` is interpreted using `unit`, and `velocity` is the desired movement speed in
        RPM.
        """
        ...

    def get_velocity(self) -> float:
        """Return the motor's estimated angular velocity in RPM.

        In some cases, this reported value may be noisy or inaccurate, especially for systems where
        accurate velocity control at high speeds is required.
        """
        ...

    def get_power(self) -> float:
        """Return the power drawn by the motor in watts."""
        ...

    def get_torque(self) -> float:
        """Return the torque output of the motor in newton-meters."""
        ...

    def get_voltage(self) -> float:
        """Return the voltage the motor is drawing in volts."""
        ...

    def get_raw_position(self) -> int:
        """Return the most recently recorded raw encoder tick data from the motor's IME.

        Gearset is not taken into consideration for this raw value.
        """
        ...

    def get_current(self) -> float:
        """Return the electrical current draw of the motor in amps."""
        ...

    def get_efficiency(self) -> float:
        """Return the efficiency of the motor on a range from 0.0 to 1.0.

        An efficiency of 1.0 means that the motor is moving electrically while drawing no
        electrical power, and an efficiency of 0.0 means that the motor is drawing power but not
        moving.
        """
        ...

    def get_current_limit(self) -> float:
        """Return the current limit for the motor in amps."""
        ...

    def get_voltage_limit(self) -> float:
        """Return the voltage limit for the motor in volts, if one has been explicitly set."""
        ...

    def get_temperature(self) -> float:
        """Return the internal temperature recorded by the motor in increments of 5 °C."""
        ...

    def set_profiled_velocity(self, velocity: int) -> None:
        """Change the output velocity for a profiled movement.

        This will have no effect if the motor is not following a profiled movement.
        """
        ...

    def reset_position(self) -> None:
        """Reset the current encoder position to zero."""
        ...

    def set_current_limit(self, limit: float) -> None:
        """Set the current limit for the motor in amps."""
        ...

    def set_voltage_limit(self, limit: float) -> None:
        """Set the voltage limit for the motor in volts."""
        ...

    def is_over_temperature(self) -> bool:
        """Return `True` if the motor's over-temperature flag is set."""
        ...

    def is_over_current(self) -> bool:
        """Return `True` if the motor's over-current flag is set."""
        ...

    def is_driver_fault(self) -> bool:
        """Return `True` if a H-bridge (motor driver) fault has occurred."""
        ...

    def is_driver_over_current(self) -> bool:
        """Return `True` if the motor's H-bridge has an over-current fault."""
        ...

    motor_type: MotorType
    """The type of the motor.

    This does not check the hardware; it returns the type the motor was created with.
    """

    def get_position(self, unit: RotationUnit) -> float:
        """Return the angular position of the motor as measured by the integrated motor encoder.

        Position measurements depend on the motor's `Gearset`, and may be reported incorrectly if
        the motor is configured with the wrong gearset variant.
        """
        ...

    def set_position(self, position: float, unit: RotationUnit) -> None:
        """Set the current encoder position to the given position without moving the motor.

        This is analogous to taring or resetting the encoder so that the new position is equal to
        the given position.
        """
        ...

    def set_direction(self, direction: Direction) -> None:
        """Set the motor to operate in a given `Direction`.

        This determines which way the motor considers to be forwards.
        """
        ...

    def get_direction(self) -> Direction:
        """Return the `Direction` of this motor."""
        ...

    def get_status(self) -> int:
        """Return the raw status flags of the motor as a bitfield."""
        ...

    def get_faults(self) -> int:
        """Return the raw fault flags of the motor as a bitfield."""
        ...

    def free(self) -> None:
        """Release this motor and free its Smart Port lock."""
        ...


class TimeUnit:
    """A unit used to represent durations of time."""

    MILLIS: ClassVar['TimeUnit']
    """Time measured in milliseconds."""

    SECOND: ClassVar['TimeUnit']
    """Time measured in seconds."""


TimeUnit.MILLIS = TimeUnit()
TimeUnit.SECOND = TimeUnit()


def monotonic_time(unit: TimeUnit = TimeUnit.SECOND) -> float:
    """Return monotonic time in the given `TimeUnit` since the Venice runtime started.

    This clock only moves forwards and is intended for measuring elapsed time, not wall-clock
    time.
    """
    ...


class Vec3:
    """A mutable three-dimensional vector.

    This type stores `x`, `y`, and `z` components as floats.
    """

    x: float
    """The x component."""

    y: float
    """The y component."""

    z: float
    """The z component."""

    def __new__(cls) -> Never:
        """`Vec3` values are returned by Venice APIs and cannot be constructed directly."""
        ...


class Quaternion:
    """A mutable quaternion.

    This type stores the vector components `x`, `y`, `z` and the scalar component `w`.
    """

    x: float
    """The x component."""

    y: float
    """The y component."""

    z: float
    """The z component."""

    w: float
    """The scalar component."""

    def __new__(cls) -> Never:
        """`Quaternion` values are returned by Venice APIs and cannot be constructed directly."""
        ...


class EulerAngles:
    """Mutable Euler angles.

    This type stores `yaw`, `pitch`, and `roll` as floats. The same values are also available
    through the aliases `z`, `y`, and `x`, respectively.
    """

    yaw: float
    """The yaw component, also available as `z`."""

    pitch: float
    """The pitch component, also available as `y`."""

    roll: float
    """The roll component, also available as `x`."""

    x: float
    """Alias for `roll`."""

    y: float
    """Alias for `pitch`."""

    z: float
    """Alias for `yaw`."""

    def __new__(cls) -> Never:
        """`EulerAngles` values are returned by Venice APIs and cannot be constructed directly."""
        ...


class InertialOrientation:
    """Represents one of six possible physical IMU orientations relative to the earth's center of gravity."""

    X_DOWN: ClassVar['InertialOrientation']
    """X-axis facing down."""

    X_UP: ClassVar['InertialOrientation']
    """X-axis facing up."""

    Y_DOWN: ClassVar['InertialOrientation']
    """Y-axis facing down."""

    Y_UP: ClassVar['InertialOrientation']
    """Y-axis facing up."""

    Z_DOWN: ClassVar['InertialOrientation']
    """Z-axis facing down (VEX logo facing up)."""

    Z_UP: ClassVar['InertialOrientation']
    """Z-axis facing up (VEX logo facing down)."""


InertialOrientation.X_DOWN = InertialOrientation()
InertialOrientation.X_UP = InertialOrientation()
InertialOrientation.Y_DOWN = InertialOrientation()
InertialOrientation.Y_UP = InertialOrientation()
InertialOrientation.Z_DOWN = InertialOrientation()
InertialOrientation.Z_UP = InertialOrientation()


class CalibrateFuture:
    """An awaitable calibration operation for an inertial sensor.

    This object is returned by `InertialSensor.calibrate`. Awaiting it waits for calibration to
    begin and then finish, or raises an error if calibration times out.
    """

    def __new__(cls) -> Never:
        """`CalibrateFuture` values are returned by Venice APIs and cannot be constructed directly."""
        ...

    def __iter__(self) -> 'CalibrateFuture':
        """Await this calibration operation until it completes."""
        ...


class InertialSensor:
    """An inertial sensor (IMU) plugged into a Smart Port.

    This class provides an interface to interact with the V5 Inertial Sensor, which combines a
    3-axis accelerometer and 3-axis gyroscope for precise motion tracking and navigation
    capabilities.

    ## Hardware Overview

    The IMU's integrated accelerometer measures linear acceleration along three axes:
    - X-axis: Forward/backward motion
    - Y-axis: Side-to-side motion
    - Z-axis: Vertical motion

    These accelerometer readings include the effect of gravity, which can be useful for determining
    the sensor's orientation relative to the ground.

    The IMU also has a gyroscope that measures rotational velocity and position on three axes:
    - Roll: Rotation around X-axis
    - Pitch: Rotation around Y-axis
    - Yaw: Rotation around Z-axis

    Like all other Smart devices, VEXos will process sensor updates every 10mS.

    ## Coordinate System

    The IMU uses a NED (North-East-Down) right-handed coordinate system:
    - X-axis: Positive towards the front of the robot (North)
    - Y-axis: Positive towards the right of the robot (East)
    - Z-axis: Positive downwards (towards the ground)

    This NED convention means that when the robot is on a flat surface:
    - The Z acceleration will read approximately +9.81 m/s² (gravity)
    - Positive roll represents clockwise rotation around the X-axis (when looking North)
    - Positive pitch represents nose-down rotation around the Y-axis
    - Positive yaw represents clockwise rotation around the Z-axis (when viewed from above)

    ## Calibration & Mounting Considerations

    The IMU requires a calibration period to establish its reference frame in one of six possible
    orientations, described by `InertialOrientation`. The sensor must be mounted flat in one of
    these orientations. Readings will be unpredictable if the IMU is mounted at an angle or was
    moving or disturbed during calibration.

    In addition, physical pressure on the sensor's housing or static electricity can cause issues
    with the onboard gyroscope, so pressure-mounting the IMU or placing the IMU low to the ground
    is undesirable.

    ## Disconnect Behavior

    If the IMU loses power due to a disconnect, even momentarily, all calibration data will be lost
    and VEXos will re-initiate calibration automatically. The robot cannot be moving when this
    occurs due to the aforementioned unpredictable behavior. As such, it is vital that the IMU
    maintain a stable connection to the Brain and voltage supply during operation.
    """

    def __init__(self, port: int) -> None:
        """Create a new inertial sensor.

        This sensor must be calibrated using `InertialSensor.calibrate` before any meaningful data
        can be read from it.
        """
        ...

    def calibrate(self) -> CalibrateFuture:
        """Calibrate the IMU.

        Await the returned `CalibrateFuture` to wait until calibration has finished or timed out.
        The sensor must remain still during calibration.
        """
        ...

    def get_heading(self, unit: RotationUnit) -> float:
        """Return the sensor's yaw angle bounded to the range `[0.0, 360.0)` in the given unit."""
        ...

    def set_heading(self, heading: float, unit: RotationUnit) -> None:
        """Set the current heading reading to the given value.

        This only affects the value returned by `InertialSensor.get_heading`.
        """
        ...

    def reset_heading(self) -> None:
        """Reset the current heading reading to zero."""
        ...

    def get_rotation(self, unit: RotationUnit) -> float:
        """Return the total amount the sensor has rotated about the z-axis in the given unit."""
        ...

    def set_rotation(self, rotation: float, unit: RotationUnit) -> None:
        """Set the current rotation reading to the given value.

        This only affects the value returned by `InertialSensor.get_rotation`.
        """
        ...

    def reset_rotation(self) -> None:
        """Reset the current rotation reading to zero."""
        ...

    def get_euler(self, unit: RotationUnit) -> EulerAngles:
        """Return Euler angles representing the inertial sensor's orientation.

        The returned values are normalized to half a turn, meaning they range from (-180°, 180°]
        in degree-based units.
        """
        ...

    def get_quaternion(self) -> Quaternion:
        """Return a quaternion representing the inertial sensor's current orientation."""
        ...

    def get_gyro_rate(self) -> Vec3:
        """Return the sensor's raw gyroscope readings in degrees per second."""
        ...

    def get_acceleration(self) -> Vec3:
        """Return the sensor's raw acceleration readings in g."""
        ...

    def is_calibrating(self) -> bool:
        """Return `True` if the sensor is currently calibrating."""
        ...

    def is_auto_calibrated(self) -> bool:
        """Return `True` if the sensor was calibrated using auto-calibration."""
        ...

    def get_physical_orientation(self) -> InertialOrientation:
        """Return the physical orientation of the sensor as measured during calibration."""
        ...

    def set_data_interval(self, interval: float, unit: TimeUnit) -> None:
        """Set the internal computation speed of the IMU.

        This does not change the rate at which user code can read data from the IMU.
        """
        ...


class RotationSensor:
    """A rotation sensor plugged into a Smart Port.

    This class provides an interface to interact with the VEX V5 Rotation Sensor, which measures
    the absolute position, rotation count, and angular velocity of a rotating shaft.

    ## Hardware Overview

    The sensor provides absolute rotational position tracking from 0° to 360° with 0.088°
    accuracy. The sensor is comprised of two magnets which utilize the [Hall Effect] to indicate
    angular position. A chip inside the rotation sensor then keeps track of the total rotations of
    the sensor to determine total position traveled.

    Position is reported by VEXos in centidegrees before being converted to the requested rotation
    unit.

    The absolute angle reading is preserved across power cycles, while the position count stores
    the cumulative forward and reverse revolutions relative to program start. However, the position
    reading will be reset if the sensor loses power. Angular velocity is measured in degrees per
    second.

    Like all other Smart devices, VEXos will process sensor updates every 10mS.

    [Hall Effect]: https://en.wikipedia.org/wiki/Hall_effect_sensor
    """

    MIN_DATA_INTERVAL_MS: ClassVar[int]
    """The minimum data rate that can be set, in milliseconds."""

    TICKS_PER_REVOLUTION: ClassVar[int]
    """The amount of unique sensor readings per one revolution of the sensor."""

    def __init__(self, port: int, direction: Direction = Direction.FORWARD) -> None:
        """Create a new rotation sensor on the given port.

        `port` must be between 1 and 21.
        """
        ...

    def get_angle(self, unit: RotationUnit) -> float:
        """Return the absolute angle of rotation measured by the sensor.

        This value is reported from 0 to 360 degrees, converted to the requested unit.
        """
        ...

    def get_position(self, unit: RotationUnit) -> float:
        """Return the total accumulated rotation of the sensor over time."""
        ...

    def set_position(self, position: float, unit: RotationUnit) -> None:
        """Set the sensor's position reading."""
        ...

    def get_velocity(self) -> float:
        """Return the sensor's current velocity in degrees per second."""
        ...

    def reset_position(self) -> None:
        """Reset the sensor's position reading to zero."""
        ...

    def set_direction(self, direction: Direction) -> None:
        """Set the sensor to operate in a given `Direction`.

        This determines which way the sensor considers to be forwards.
        """
        ...

    def get_direction(self) -> Direction:
        """Return the `Direction` of this sensor."""
        ...

    def get_status(self) -> int:
        """Return the sensor's internal status code as a bitfield."""
        ...

    def set_data_interval(self, interval: float, unit: TimeUnit) -> None:
        """Set the internal computation speed of the rotation sensor.

        This does not change the rate at which user code can read data from the sensor.
        """
        ...

    def free(self) -> None:
        """Release this rotation sensor and free its Smart Port lock."""
        ...


RotationSensor.MIN_DATA_INTERVAL_MS = 5
RotationSensor.TICKS_PER_REVOLUTION = 36000

###########################################
# Binary file provider                    #
# Stuff for the CLI. DO NOT use in user   #
# code                                    #
###########################################
import importlib.resources as pkg_resources  # noqa: E402
import importlib.metadata  # noqa: E402

__version__: str
try:
    __version__ = importlib.metadata.version(__package__ if __package__ else "")
except importlib.metadata.PackageNotFoundError:
    __version__ = "0.1.0"

def _dangerous_DO_NOT_TOUCH_YOU_WILL_GET_ELECTROCUTED_get_binary_path():  # pyright: ignore[reportUnusedFunction]
    """DO NOT use this method in user programs that you are intending to run on the VEX V5.

    Internal function for use by the CLI."""
    return pkg_resources.files("venice").joinpath("venice.bin")
def _dangerous_DO_NOT_TOUCH_YOU_WILL_GET_ELECTROCUTED_get_version():  # pyright: ignore[reportUnusedFunction]
    """DO NOT use this method in user programs that you are intending to run on the VEX V5.

    Internal function for use by the CLI."""
    return __version__
