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
from typing import ClassVar

class RotationUnit:
    """Represents a unit of rotation for angles."""

    RADIANS: ClassVar['RotationUnit']
    """The rotation unit representing radians."""

    DEGREES: ClassVar['RotationUnit']
    """The rotation unit representing degrees."""

    TURNS: ClassVar['RotationUnit']
    """The rotation unit representing turns (revolutions)."""

RotationUnit.RADIANS = RotationUnit()
RotationUnit.DEGREES = RotationUnit()
RotationUnit.TURNS = RotationUnit()

class TimeUnit:
    """Represents a unit of time."""

    MILLIS: ClassVar['TimeUnit']
    """The time unit representing milliseconds."""

    SECONDS: ClassVar['TimeUnit']
    """The time unit representing seconds."""

TimeUnit.MILLIS = TimeUnit()
TimeUnit.SECONDS = TimeUnit()


class Direction:
    """A rotational direction."""

    FORWARD: ClassVar['Direction']
    """Rotates in the forward direction."""

    REVERSE: ClassVar['Direction']
    """Rotates in the reverse direction."""

Direction.FORWARD = Direction()
Direction.REVERSE = Direction()

class RotationSensor:
    """A rotation sensor plugged into a Smart Port.

    The VEX V5 Rotation Sensor, measures the absolute position, rotation count,
    and angular velocity of a rotating shaft.

    # Hardware Overview

    The sensor provides absolute rotational position tracking from 0° to 360° with 0.088° accuracy.
    The sensor is compromised of two magnets which utilize the
    [Hall Effect](https://en.wikipedia.org/wiki/Hall_effect_sensor) to indicate angular
    position. A chip inside the rotation sensor (a Cortex M0+) then keeps track of the total
    rotations of the sensor to determine total position traveled.

    Position is reported by VEXos in centidegrees before being converted to an float
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

        Returns:
            The total accumulated rotation of the sensor in the specified units.
        """
        ...

    def angle(self, unit: RotationUnit) -> float:
        """Returns the absolute angle of rotation measured by the sensor.

        This value is reported from 0-360 degrees.

        Args:

        * unit: The `RotationUnit` to use for the return value.

        Returns:
            The absolute angle of rotation measured by the sensor in the
            specified units.
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
