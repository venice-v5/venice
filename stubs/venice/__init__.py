"""
Venice is an open-source Micropython runtime for VEX V5 robots.

```python
from venice import Motor, Gearset, Direction
import venice.vasyncio

async def main():
    my_motor = Motor(
        1,
        Direction.FORWARD,
        Gearset.GREEN
    )
    my_motor.set_voltage(10.0)
vasyncio.run(main())
```
"""

from __future__ import annotations

from typing import ClassVar, List, Literal, Union


class RotationUnit:
    """Represents a unit of rotation for angles."""

    RADIANS: ClassVar["RotationUnit"]
    """The rotation unit representing radians."""

    DEGREES: ClassVar["RotationUnit"]
    """The rotation unit representing degrees."""

    TURNS: ClassVar["RotationUnit"]
    """The rotation unit representing turns (revolutions)."""


RotationUnit.RADIANS = RotationUnit()
RotationUnit.DEGREES = RotationUnit()
RotationUnit.TURNS = RotationUnit()


class TimeUnit:
    """Represents a unit of time."""

    MILLIS: ClassVar["TimeUnit"]
    """The time unit representing milliseconds."""

    SECONDS: ClassVar["TimeUnit"]
    """The time unit representing seconds."""


TimeUnit.MILLIS = TimeUnit()
TimeUnit.SECONDS = TimeUnit()


class Direction:
    """A rotational direction."""

    FORWARD: ClassVar["Direction"]
    """Rotates in the forward direction."""

    REVERSE: ClassVar["Direction"]
    """Rotates in the reverse direction."""


Direction.FORWARD = Direction()
Direction.REVERSE = Direction()


class Gearset:
    """Internal gearset used by VEX Smart motors."""

    RED: ClassVar["Gearset"]
    """36:1 gear ratio"""

    GREEN: ClassVar["Gearset"]
    """18:1 gear ratio"""

    BLUE: ClassVar["Gearset"]
    """6:1 gear ratio"""


Gearset.RED = Gearset()
Gearset.GREEN = Gearset()
Gearset.BLUE = Gearset()


class BrakeMode:
    """Determines the behavior a motor should use when braking with `AbstractMotor.brake`."""

    COAST: ClassVar["BrakeMode"]
    """Motor never brakes."""

    BRAKE: ClassVar["BrakeMode"]
    """Motor uses regenerative braking to slow down faster."""

    HOLD: ClassVar["BrakeMode"]
    """Motor exerts force holding itself in the same position."""


BrakeMode.COAST = BrakeMode()
BrakeMode.BRAKE = BrakeMode()
BrakeMode.HOLD = BrakeMode()


class MotorType:
    """Represents the type of a Smart motor.

    Either a 11W (V5) or 5.5W (EXP) motor."""

    EXP: ClassVar["MotorType"]
    """A 5.5W Smart Motor"""

    V5: ClassVar["MotorType"]
    """An 11W Smart Motor"""


class RotationSensor:
    """A rotation sensor plugged into a Smart Port.

    The VEX V5 Rotation Sensor, measures the absolute position, rotation count,
    and angular velocity of a rotating shaft.

    # Hardware Overview

    The sensor provides absolute rotational position tracking from 0° to 360° with 0.088° accuracy.
    The sensor is composed of two magnets which utilize the
    [Hall Effect](https://en.wikipedia.org/wiki/Hall_effect_sensor) to indicate angular
    position. A chip inside the rotation sensor (a Cortex M0+) then keeps track of the total
    rotations of the sensor to determine total position traveled.

    Position is reported by VEXos in centidegrees before being converted to a float
    in the given unit of rotation.

    The absolute angle reading is preserved across power cycles (similar to a potentiometer), while
    the position count stores the cumulative forward and reverse revolutions relative to program
    start, however the *position* reading will be reset if the sensor loses power. Angular velocity
    is measured in degrees per second.

    Like all other Smart devices, VEXos will process sensor updates every 10mS.
    """

    def __init__(self, port: int, direction: Direction = Direction.FORWARD):
        """Creates a new rotation sensor on the given port.

        Whether or not the sensor should be reversed on creation can be specified.

        Args:

        * port: The port number (1-21).

        * direction: The direction of rotation. Defaults to forward.
        """

    MIN_DATA_INTERVAL_MS: int = 5
    """The minimum data rate that you can set a rotation sensor to, in milliseconds."""

    TICKS_PER_ROTATION: int = 36000
    """The amount of unique sensor readings per one revolution of the sensor."""

    def position(self, unit: RotationUnit) -> float:
        """Returns the total accumulated rotation of the sensor over time, in
        the specified units.

        Args:

        * unit: The `RotationUnit` to use for the return value.
        """
        ...

    def angle(self, unit: RotationUnit) -> float:
        """Returns the absolute angle of rotation measured by the sensor.

        This value is reported from 0-360 degrees.

        Args:

        * unit: The `RotationUnit` to use for the return value.
        """
        ...

    def set_position(self, angle: float, angle_unit: RotationUnit):
        """Sets the sensor's position reading.

        Args:

        * angle: The angle to set the sensor to.
        * angle_unit: The `RotationUnit` to use for the angle.
        """
        ...

    def velocity(self) -> float:
        """Returns the sensor's current velocity in degrees per second."""
        ...

    def reset_position(self):
        """Resets the sensor's position reading to zero."""
        ...

    def set_direction(self, new_direction: Direction):
        """Sets the sensor to operate in a given `Direction`.

        This determines which way the sensor considers to be “forwards”. You can use the marking on
        the top of the motor as a reference:

        - When `Direction.FORWARD` is specified, positive velocity/voltage values will cause the
          motor to rotate **with the arrow on the top**. Position will increase as the motor rotates
          **with the arrow**.
        - When `Direction.REVERSE` is specified, positive velocity/voltage values will cause the
          motor to rotate **against the arrow on the top**. Position will increase as the motor
          rotates **against the arrow**.

        Args:

        * new_direction: The new `Direction` to set the sensor to.
        """
        ...

    def direction(self):
        """Returns the `Direction` of this sensor."""
        ...

    def status(self) -> int:
        """Returns the sensor's internal status code."""
        ...

    def set_data_interval(self, interval: float, interval_unit: TimeUnit):
        """Sets the internal computation speed of the rotation sensor.

        This method does NOT change the rate at which user code can read data off the sensor, as the
        brain will only talk to the device every 10mS regardless of how fast data is being sent or
        computed.

        This duration should be above `RotationSensor.MIN_DATA_INTERVAL_MS` (5 milliseconds).

        Args:

        * interval: The new interval to set the sensor to.
        * interval_unit: The unit of the interval.
        """


class AbstractMotor:
    """A motor plugged into a Smart Port.

    This is an abstract class supporting shared methods for both the 5W (Exp)
    and 11W motor variants. To create a motor, use the initializers of `ExpMotor`
    or `V5Motor`, respectively.

    # Overview

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

    # Current Limitations

    There are some cases where VEXos or the motor itself may decide to limit output current:

    - **Stall Prevention**: The stall current on 11W motors is limited to 2.5A. This limitation
      eliminates the need for automatic resetting fuses (PTC devices) in the motor, which can
      disrupt operation. By restricting the stall current to 2.5A, the motor effectively avoids
      undesirable performance dips and ensures that users do not inadvertently cause stall
      situations.

    - **Motor Count**: Robots that use 8 or fewer 11W motors will have the aforementioned current limit
      of 2.5A set for each motor. Robots using more than 8 motors, will have a lower default current limit
      per-motor than 2.5A. This limit is determined in VEXos by a calculation accounting for the number of
      motors plugged in, and the user's manually set current limits using `AbstractMotor.set_current_limit`. For
      more information regarding the current limiting behavior of VEXos, see [this forum post](https://www.vexforum.com/t/how-does-the-decreased-current-affect-the-robot-when-using-more-than-8-motors/72650/4).

    - **Temperature Management**: Motors have an onboard sensor for measuring internal temperature.
      If the motor determines that it is overheating, it will throttle its output current and warn
      the user.

    # Motor Control

    Each motor contains a sophisticated control system built around a Cortex M0+ microcontroller.
    The microcontroller continuously monitors position, speed, direction, voltage, current, and
    temperature through integrated sensors.

    The onboard motor firmware implements a set of pre-tuned PID (Proportional-Integral-Derivative)
    controllers operating on a 10-millisecond cycle for position and velocity control. Motors also
    have braking functionality for holding a specific position under load.
    """

    WRITE_INTERVAL_MS: int = 5
    """The interval at which the Brain will send new packets to an `AbstractMotor`."""

    def set_voltage(self, voltage: float):
        """Sets the motor's output voltage.

        This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards)
        volts, and controls the raw output of the motor.

        Args:

        - voltage: The output voltage of the motor in volts.
        """
        ...

    def set_velocity(self, rpm: int):
        """Spins the motor at a target velocity.

        This velocity corresponds to different actual speeds in RPM depending on the gearset used
        for the motor. Velocity is held with an internal PID controller to ensure consistent
        speed, as opposed to setting the motor's voltage.

        Args:

        - rpm: The desired target velocity in RPM.
        """
        ...

    def brake(self):
        """Sets the motor's target to a given `BrakeMode`."""
        ...

    def set_position_target(
        self, position: float, position_unit: RotationUnit, velocity: int
    ):
        """Sets an absolute position target for the motor to attempt to reach using its internal
        PID control.

        Args:

        - position: The desired position of the motor after the movement operation.
        - position_unit: The unit of the position.
        - velocity: The desired speed of the velocity in RPM during the movement operation
        """
        ...

    def is_exp(self) -> bool:
        """Returns `True` if the motor is a 5.5W (EXP) Smart Motor."""
        ...

    def is_v5(self) -> bool:
        """Returns `True` if the motor is an 11W (V5) Smart Motor."""
        ...

    def max_voltage(self) -> float:
        """Returns the maximum voltage for the motor based off of its `MotorType`.

        See `V5Motor.MAX_VOLTAGE` and `ExpMotor.MAX_VOLTAGE`."""
        ...

    def velocity(self) -> float:
        """Returns the motor's estimate of its angular velocity in rotations per minute (RPM).

        # Accuracy

        In some cases, this reported value may be noisy or inaccurate, especially for systems where
        accurate velocity control at high speeds is required (such as flywheels). If the
        accuracy of this value proves inadequate, you may opt to perform your own velocity
        calculations by differentiating `AbstractMotor.position` over the reported internal timestamp
        of the motor (currently not exposed in the Venice API).

        > For more information about Smart motor velocity estimation, see [this article](https://sylvie.fyi/sylib/docs/db/d8e/md_module_writeups__velocity__estimation.html).

        # Note

        This is the **estimated** velocity, not the **target** velocity. The Venice API currently does
        not mirror vexide's `Motor::target` so there is no way to get the target velocity.
        """
        ...

    def power(self) -> float:
        """Returns the power drawn by the motor in Watts."""
        ...

    def torque(self) -> float:
        """Returns the torque output of the motor in Nm."""
        ...

    def voltage(self) -> float:
        """Returns the voltage the motor is drawing in volts."""
        ...

    def raw_position(self) -> int:
        """Returns the most recently recorded raw encoder tick data from the motor's IME.

        The motor's integrated encoder has a TPR of 4096. Gearset is not taken into consideration
        when dealing with the raw value, meaning this measurement will be taken relative to the
        motor's internal position *before* being geared down from 3600RPM.

        Methods such as `AbstractMotor.reset_position` and `AbstractMotor.set_position` do not
        change the value of this raw measurement.
        """
        ...

    def current(self) -> float:
        """Returns the electrical current draw of the motor in amps."""
        ...

    def efficiency(self) -> float:
        """Returns the efficiency of the motor from a range of [0.0, 1.0].

        An efficiency of 1.0 means that the motor is moving electrically while drawing no electrical
        power, and an efficiency of 0.0 means that the motor is drawing power but not moving."""
        ...

    def current_limit(self) -> float:
        """Returns the current limit for the motor in amps.

        This limit can be configured with the `AbstractMotor.set_current_limit` method."""
        ...

    def voltage_limit(self) -> float:
        """Returns the voltage limit for the motor if one has been explicitly set."""
        ...

    def temperature(self) -> float:
        """Returns the internal temperature recorded by the motor in increments of 5 °C."""
        ...

    def set_profiled_velocity(self, rpm: int):
        """Changes the output velocity for a profiled movement (motor_move_absolute or
        motor_move_relative).

        This will have no effect if the motor is not following a profiled movement.

        Args:

        - rpm: The new profiled velocity target in rotations per minute.
        """
        ...

    def reset_position(self):
        """Sets the current encoder position to zero without moving the motor.

        Analogous to taring or resetting the encoder to the current position.
        """
        ...

    def set_current_limit(self, amps: float):
        """Sets the current limit for the motor in amps.

        Args:

        - amps: The new current limit in amps.
        """
        ...

    def set_voltage_limit(self, volts: float):
        """Sets the voltage limit for the motor in volts.

        Args:

        - volts: The new voltage limit in volts.
        """
        ...

    def is_over_temperature(self) -> bool:
        """Returns `True` if the motor's over temperature flag is set."""
        ...

    def is_over_current(self):
        """Returns `True` if the motor's over-current flag is set."""
        ...

    def is_driver_fault(self):
        """Returns `True` if a H-bridge (motor driver) fault has occurred."""
        ...

    def is_driver_over_current(self):
        """Returns `True` if the motor's H-bridge has an over-current fault."""
        ...

    def status(self) -> int:
        """Returns the status flags of a motor as bits."""
        ...

    def faults(self) -> int:
        """Returns the fault flags of the motor as bits."""
        ...

    def motor_type(self):
        """Returns the type of the motor.

        This does not check the hardware, it simply returns the type that the motor was created
        with."""
        ...

    def position(self, unit: RotationUnit) -> float:
        """Returns the angular position of the motor as measured by the IME (integrated motor encoder).

        This is returned as a floating-point number in the given `RotationUnit`.

        # Gearing affects position!

        Position measurements are dependent on the Motor's `Gearset`, and may be reported
        incorrectly if the motor is configured with the incorrect gearset variant. Make sure
        that the motor is configured with the same gearset as its physical cartridge color.

        Args:

        - unit: The unit in which to return the position.
        """
        ...

    def set_position(self, position: float, position_unit: RotationUnit):
        """Sets the current encoder position to the given position without moving the motor.

        Analogous to taring or resetting the encoder so that the new position is equal to the given
        position.

        Args:

        - position: The position to set the encoder to.
        - position_unit: The unit of the given position.
        """
        ...

    def set_direction(self, direction: Direction):
        """Sets the motor to operate in a given `Direction`.

        This determines which way the motor considers to be “forwards”. You can use the marking on
        the back of the motor as a reference:

        - When `Direction.FORWARD` is specified, positive velocity/voltage values will cause the
          motor to rotate **with the arrow on the back**. Position will increase as the motor
          rotates **with the arrow**.
        - When `Direction.REVERSE` is specified, positive velocity/voltage values will cause the
          motor to rotate **against the arrow on the back**. Position will increase as the motor
          rotates **against the arrow**.

        Args:

        - direction: The direction to set the motor to.
        """
        ...

    def direction(self):
        """Returns the `Direction` of this motor."""
        ...


class V5Motor(AbstractMotor):
    """Represents an 11W (V5) Smart Motor. See `AbstractMotor`."""

    MAX_VOLTAGE: Literal[12] = 12
    """The maximum voltage value that can be sent to a `V5Motor`."""

    def __init__(self, port: int, direction: Direction, gearset: Gearset):
        """Creates a new 11W (V5) Smart Motor.

        See `ExpMotor.__init__` to create a 5.5W (EXP) Smart Motor.

        Args:

        - port: The smart port to initialize the motor on.
        - direction: The direction to set the motor to.
        - gearset: The gearset to set the motor to.
        """
        ...

    def set_gearset(self, gearset: Gearset):
        """Sets the gearset of an 11W motor.

        This may silently fail if the motor is a 5.5W motor, which occurs in the edge
        case described in `V5Motor.gearset`.

        Args:

        - gearset: the new `Gearset` to use for the motor.
        """
        ...

    def gearset(self) -> Gearset:
        """Returns the gearset of an 11W motor.

        For 5.5W motors [1], this will always be returned as `Gearset.GREEN`.

        [1] There is a slim edge case in which you initialize a `V5Motor` at a time
        when the specified port indeed has a plugged-in 11W motor, but then unplug
        it and replace it with a 5.5W motor. Note that the respective constructors of
        `V5Motor` and `ExpMotor` *will* throw if the motor is not the correct type.
        """
        ...


class ExpMotor(AbstractMotor):
    """Represents an 5.5W (EXP) Smart Motor. See `AbstractMotor`."""

    def __init__(self, port: int, direction: Direction):
        """Creates a new 5.5W (EXP) Smart Motor.

        See `V5Motor.__init__` to create a 11W (V5) Smart Motor.

        Args:

        - port: The smart port to initialize the motor on.
        - direction: The direction to set the motor to.
        """
        ...

    MAX_VOLTAGE: Literal[8] = 8
    """The maximum voltage value that can be sent to a `ExpMotor`."""


class DistanceObject:
    """Readings from a physical object detected by a Distance Sensor."""

    distance: int
    "The distance of the object from the sensor (in millimeters)."

    confidence: float
    """The confidence in the distance measurement from 0.0 to 1.0."""

    velocity: float
    """Observed velocity of the object in m/s."""

    relative_size: int
    """A guess at the object's "relative size".

    This is a value that has a range of 0 to 400. A 18" x 30" grey card will return a value of
    approximately 75 in typical room lighting. If the sensor is not able to detect an object,
    None is returned.

    This sensor reading is unusual, as it is entirely unitless with the seemingly arbitrary
    range of 0-400 existing due to VEXCode's
    [`vex::sizeType`](https://api.vex.com/v5/home/python/Enums.html#object-size-types) enum
    having four variants. It's unknown what the sensor is *actually* measuring here either,
    so use this data with a grain of salt."""


class DistanceSensor:
    """A distance sensor plugged into a Smart Port.

    The VEX V5 Distance Sensor uses a
    Class 1 laser to measure the distance, object size classification, and relative velocity of a
    single object.

    # Hardware Overview

    The sensor uses a narrow-beam Class 1 laser (similar to phone proximity sensors) for precise
    detection. It measures distances from 20mm to 2000mm with varying accuracy (±15mm below 200mm,
    ±5% above 200mm).

    The sensor can classify detected objects by relative size, helping distinguish between walls and
    field elements. It also measures the relative approach velocity between the sensor and target.

    Due to the use of a laser, measurements are single-point and highly directional, meaning that
    objects will only be detected when they are directly in front of the sensor's field of view.

    Like all other Smart devices, VEXos will process sensor updates every 10mS.
    """

    def __init__(self, port: int):
        """Creates a new distance sensor given the port.

        Args:

        - port: The port number of the sensor.
        """
        ...

    def object(self) -> Union[DistanceObject, None]:
        """Attempts to detect an object, returning `None` if no object could be found."""
        ...

    def status(self) -> int:
        """Returns the internal status code of the distance sensor.

        The status code of the signature can tell you if the sensor is still initializing or if it
        is working correctly.

        - If the distance sensor is still initializing, the status code will be 0x00.
        - If it is done initializing and functioning correctly, the status code will be 0x82 or
          0x86.
        """
        ...


class AiVisionColor:
    """A color signature used by an AI Vision Sensor to detect color blobs."""

    r: int
    """The red component of the RGB color value."""

    g: int
    """The green component of the RGB color value."""

    b: int
    """The blue component of the RGB color value."""

    hue_range: float
    """The accepted hue range of the color. VEXcode limits this value to [0, 20]"""

    saturation_range: float
    """The accepted saturation range of the color."""

    def __init__(
        self, r: int, g: int, b: int, hue_range: float, saturation_range: float
    ):
        """Creates a new `AiVisionColor` from the provided args.

        Args:

        - r: The red component of the RGB color value.
        - g: The green component of the RGB color value.
        - b: The blue component of the RGB color value.
        - hue_range: The accepted hue range of the color.
        - saturation_range: The accepted saturation range of the color.
        """
        ...


class AiVisionColorCode:
    """A color code used by an AI Vision Sensor to detect groups of color blobs.

    Color codes are effectively "groups" of color signatures. A color code associated multiple color
    signatures on the sensor will be detected as a single object when all signatures are seen next
    to each other.

    Color codes can associate up to 7 color signatures and detections will be returned as
    `AiVisionObject.Code` variants.
    """

    def __init__(
        self,
        first: Union[int, None],
        second: Union[int, None],
        third: Union[int, None],
        fourth: Union[int, None],
        fifth: Union[int, None],
        sixth: Union[int, None],
        seventh: Union[int, None],
    ):
        """Creates a new `AiVisionColorCode` from the provided args.

        Args:

        - first: the first color signature id.
        - second: the second color signature id.
        - third: the third color signature id.
        - fourth: the fourth color signature id.
        - fifth: the fifth color signature id.
        - sixth: the sixth color signature id.
        - seventh: the seventh color signature id.
        """
        ...

    # we annotate the docstring as @public because pdoc otherqise doesn't
    # document slot items
    def __getitem__(self, item: int):
        """@public Returns the color signature id at the given index.

        Note that this does NOT support setting. For example, the following code works:
        ```python
        color_code_ids = [1, 2, 3, 4, 5, 6, 7]
        my_color_code = AiVisionColorCode(*color_code_ids)
        print(my_color_code[2]) # prints 3
        ```
        But this code doesn't:
        ```python
        color_code_ids = [1, 2, 3, 4, 5, 6, 7]
        my_color_code = AiVisionColorCode(*color_code_ids)
        my_color_code[6] = 9 # fails
        ```
        """
        ...


class AiVisionDetectionMode:
    """Flags relating to the sensor's detection mode."""

    APRILTAG: ClassVar["AiVisionDetectionMode"]
    """Enable apriltag detection"""

    COLOR: ClassVar["AiVisionDetectionMode"]
    """Enable color detection"""

    MODEL: ClassVar["AiVisionDetectionMode"]
    """Enable model detection"""

    COLOR_MERGE: ClassVar["AiVisionDetectionMode"]
    """Merge color blobs?"""

    def __or__(self, value) -> AiVisionDetectionMode:
        """@public Bitwise OR is applied to the internal bitflags, "merging" the modes. The result is returned

        ```python
        sensor = AiVisionSensor(7)
        # enable apriltag and color detection
        sensor.set_detection_mode(AiVisionDetectionMode.APRILTAG | AiVisionDetectionMode.COLOR)
        ```
        """
        ...

    def __ior__(self, value):
        """@public Bitwise OR is applied to the internal bitflags, "merging" the modes. `self` is set to the result.

        ```python
        sensor = AiVisionSensor(7)
        # enable apriltag and color detection
        mode = AiVisionDetectionMode.APRILTAG
        mode |= AiVisionDetectionMode.COLOR
        sensor.set_detection_mode(mode)
        ```
        """
        ...


AiVisionDetectionMode.APRILTAG = AiVisionDetectionMode()
AiVisionDetectionMode.COLOR = AiVisionDetectionMode()
AiVisionDetectionMode.MODEL = AiVisionDetectionMode()
AiVisionDetectionMode.COLOR_MERGE = AiVisionDetectionMode()


class AiVisionFlags:
    """Represents the mode of the AI Vision sensor."""

    DISABLE_APRILTAG: ClassVar["AiVisionFlags"]
    """Disable apriltag detection"""

    DISABLE_COLOR: ClassVar["AiVisionFlags"]
    """Disable color detection"""

    DISABLE_MODEL: ClassVar["AiVisionFlags"]
    """Disable model detection"""

    COLOR_MERGE: ClassVar["AiVisionFlags"]
    """Merge color blobs?"""

    DISABLE_STATUS_OVERLAY: ClassVar["AiVisionFlags"]
    """Disable status overlay"""

    DISABLE_USB_OVERLAY: ClassVar["AiVisionFlags"]
    """Disable USB overlay"""

    def __or__(self, value) -> AiVisionDetectionMode:
        """@public Bitwise OR is applied to the internal bitflags, "merging" the modes. The result is returned

        ```python
        sensor = AiVisionSensor(7)
        # disable status and usb overlays
        sensor.set_flags(AiVisionFlags.DISABLE_STATUS_OVERLAY | AiVisionFlags.DISABLE_USB_OVERLAY)
        ```
        """
        ...

    def __ior__(self, value):
        """@public Bitwise OR is applied to the internal bitflags, "merging" the modes. `self` is set to the result.

        ```python
        sensor = AiVisionSensor(7)
        # disable status and usb overlays
        mode = AiVisionFlags.DISABLE_STATUS_OVERLAY
        mode |= AiVisionFlags.DISABLE_USB_OVERLAY
        sensor.set_flags(mode)
        ```
        """
        ...


AiVisionFlags.DISABLE_APRILTAG = AiVisionFlags()
AiVisionFlags.DISABLE_COLOR = AiVisionFlags()
AiVisionFlags.DISABLE_MODEL = AiVisionFlags()
AiVisionFlags.COLOR_MERGE = AiVisionFlags()
AiVisionFlags.DISABLE_STATUS_OVERLAY = AiVisionFlags()
AiVisionFlags.DISABLE_USB_OVERLAY = AiVisionFlags()


class AiVisionColorObject:
    """The data associated with an AI Vision object detected by color blob detection."""

    id: int
    """ID of the signature used to detect this object."""

    position_x: int
    """The x component of the top-left corner of the object."""

    position_y: int
    """The y component of the top-left corner of the object."""

    width: int
    """The width of the object."""

    height: int
    """The height of the object."""


class AiVisionCodeObject:
    """The data associated with an AI Vision object detected by color code detection."""

    id: int
    """ID of the code used to detect this object."""

    position_x: int
    """The x component of the position of the object."""

    position_y: int
    """The y component of the position of the object."""

    width: int
    """The width of the object."""

    height: int
    """The height of the object."""

    def angle(self, unit: RotationUnit) -> float:
        """The angle of the object's associated colors. Not always reliably available.

        Args:

        - units: The unit of measurement for the angle. This is the unit of
        the returned `float`.
        """
        ...


class AiVisionAprilTagObject:
    """The data associated with an AI Vision object detected by apriltag detection."""

    id: int
    """The detected AprilTag(s) ID number"""

    top_left_x: int
    """x component of the position of the top-left corner of the tag"""

    top_left_y: int
    """y component of the position of the top-left corner of the tag"""

    top_right_x: int
    """x component of the position of the top-right corner of the tag"""

    top_right_y: int
    """y component of the position of the top-right corner of the tag"""

    bottom_left_x: int
    """x component of the position of the bottom-left corner of the tag"""

    bottom_left_y: int
    """y component of the position of the bottom-left corner of the tag"""

    bottom_right_x: int
    """x component of the position of the bottom-right corner of the tag"""

    bottom_right_y: int
    """y component of the position of the bottom-right corner of the tag"""


class AiVisionModelObject:
    """The data associated with an AI Vision object detected by an onboard model."""

    id: int
    """ID of the detected object."""

    position_x: int
    """The x component of the position of the object."""

    position_y: int
    """The y component of the position of the object."""

    width: int
    """The width of the object."""

    height: int
    """The height of the object."""

    confidence: int
    """The confidence reported by the model."""

    classification: str
    """A string describing the specific onboard model used to detect this object."""


class AprilTagFamily:
    """Possible april tag families to be detected by the `AiVisionSensor`."""

    CIRCLE21H7: ClassVar["AprilTagFamily"]
    """Circle21h7 family"""

    TAG16H5: ClassVar["AprilTagFamily"]
    """16h5 family"""

    TAG36H11: ClassVar["AprilTagFamily"]
    """36h11 family"""

    TAG25H9: ClassVar["AprilTagFamily"]
    """25h9 family"""


AprilTagFamily.TAG36H11 = AprilTagFamily()
AprilTagFamily.TAG25H9 = AprilTagFamily()
AprilTagFamily.TAG16H5 = AprilTagFamily()
AprilTagFamily.CIRCLE21H7 = AprilTagFamily()


class AiVisionSensor:
    """An AI Vision sensor.

    The AI Vision sensor is
    meant to be a direct upgrade from the Vision Sensor with a wider camera range
    and AI model capabilities.

    # Hardware overview

    The AI Vision sensor has three detection modes that can all be enabled at the same time:
    - [Color detection](AiVisionDetectionMode::COLOR)
    - [Custom model detection](AiVisionDetectionMode::MODEL)
    - [AprilTag detection](AiVisionDetectionMode::APRILTAG) (requires color detection to be enabled)

    Currently there is no known way to upload custom models to the sensor and fields do not have
    AprilTags. However, there are built-in models that can be used for detection.

    See [VEX's documentation](https://kb.vex.com/hc/en-us/articles/30326315023892-Using-AI-Classifications-with-the-AI-Vision-Sensor) for more information.

    The resolution of the AI Vision sensor is 320x240 pixels. It has a horizontal FOV of 74 degrees
    and a vertical FOV of 63 degrees. Both of these values are a slight upgrade from the Vision
    Sensor.

    Unlike the Vision Sensor, the AI Vision sensor uses more human-readable color signatures that
    may be created without the AI Vision utility, though uploading color signatures with VEX's AI
    Vision Utility over USB is still an option.
    """

    MAX_OBJECTS: int = 24
    """Maximum number of objects that can be detected at once."""

    HORIZONTAL_RESOLUTION: int = 320
    """The horizontal resolution of the AI Vision sensor."""

    VERTICAL_RESOLUTION: int = 240
    """The vertical resolution of the AI Vision sensor."""

    HORIZONTAL_FOV: float = 74.0
    """The horizontal FOV of the vision sensor in degrees."""

    VERTICAL_FOV: float = 63.0
    """The vertical FOV of the vision sensor in degrees."""

    DIAGONAL_FOV: float = 87.0
    """The diagonal FOV of the vision sensor in degrees."""

    def temperature(self) -> float:
        """Returns the current temperature of the sensor in degrees Celsius."""
        ...

    def set_color_code(self, id: int, code: AiVisionColorCode):
        """
        Registers a color code association on the sensor.

        Color codes are effectively "groups" of color signatures. A color code associated multiple
        color signatures on the sensor will be detected as a single object when all signatures are
        seen next to each other.

        Args:

        - id: The ID of the color code to register. Must be in [1, 8].
        - code: The `AiVisionColorCode` to register.
        """

    def color_code(self, id: int) -> Union[AiVisionColorCode, None]:
        """Returns the color code set on the AI Vision sensor with the given ID if it exists.

        Args:

        - id: The ID of the color code to retrieve. Must be in [1, 8].

        Returns:

        - The color code associated with the given ID if it exists, or None if no color code is set.
        """
        ...

    def color_codes(self) -> List[Union[AiVisionColorCode, None]]:
        """Returns all color codes set on the AI Vision sensor as a list of 8 optional `AiVisionColorCode` objects."""
        ...

    def set_color(self, id: int, color: AiVisionColor) -> AiVisionColor:
        """Sets a color signature for the AI Vision sensor.

        Args:

        - id: The ID of the color signature to register. Must be in [1, 7].
        - color: The `AiVisionColor` to register.
        """
        ...

    def color(self, id: int) -> Union[AiVisionColor, None]:
        """Returns the color signature set on the AI Vision sensor with the given ID if it exists.

        Args:

        - id: The ID of the color signature to retrieve. Must be in [1, 7].
        """
        ...

    def colors(self) -> List[Union[AiVisionColor, None]]:
        """Returns all color signatures set on the AI Vision sensor as a list of 7 optional `AiVisionColor` objects."""
        ...

    def set_detection_mode(self, mode: AiVisionDetectionMode):
        """Sets the detection mode of the AI Vision sensor.

        Args:

        - mode: The new `AiVisionDetectionMode` to set.
        """
        ...

    def flags(self) -> AiVisionFlags:
        """Returns the current flags of the AI Vision sensor including the detection mode flags set by `AiVisionSensor.set_detection_mode`."""
        ...

    def set_flags(self, mode: AiVisionFlags):
        """Set the full flags of the AI Vision sensor, including the detection mode.

        Args:

        - mode: The new `AiVisionFlags` to set.
        """
        ...

    def start_awb(self):
        """Restarts the automatic white balance process."""
        ...

    def enable_test(self, test: int):
        """Unknown use.

        Args:

        - test: unknown purpose.
        """
        ...

    def set_apriltag_family(self, family: AprilTagFamily):
        """Sets the AprilTag family that the sensor will try to detect.

        Args:

        - family: The `AprilTagFamily` for the sensor to try to detect.
        """
        ...

    def object_count(self) -> int:
        """Returns the number of objects currently detected by the AI Vision sensor."""
        ...

    def objects(
        self,
    ) -> List[
        Union[
            AiVisionColorObject,
            AiVisionCodeObject,
            AiVisionModelObject,
            AiVisionAprilTagObject,
        ]
    ]:
        """Returns all objects detected by the AI Vision sensor."""
        ...


# TODOs:
# * controller
# * vasyncio
# * competition runtime
