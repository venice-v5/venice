"""
Venice is an open-source Micropython runtime for VEX V5 robots.

```python
from venice import Direction, Gearset, Motor, Sleep, TimeUnit, run

async def main():
    my_motor = Motor(
        1,
        Direction.FORWARD,
        Gearset.GREEN
    )
    my_motor.set_voltage(10.0)

    while True:
        await Sleep(50, TimeUnit.MILLIS)

run(main())
```
"""

from __future__ import annotations

from typing import Any, Awaitable, ClassVar, Generic, List, Literal, TypeVar, Union, Never

_T = TypeVar('_T')

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

    def is_exp(self) -> bool:
        """Return `True` if the motor is a 5.5W (EXP) Smart Motor."""
        ...

    def is_v5(self) -> bool:
        """Return `True` if the motor is an 11W (V5) Smart Motor."""
        ...

    def get_max_voltage(self) -> float:
        """Return the maximum voltage for the motor based on its motor type."""
        ...

    def gearset(self) -> Gearset:
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

    def get_motor_type(self) -> MotorType:
        """Return the type of the motor.

        This does not check the hardware; it returns the type the motor was created with.
        """
        ...

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


class Task(Generic[_T]):
    """A scheduled asynchronous task.

    Tasks are created by `EventLoop.spawn` or `spawn`. They wrap a coroutine that will be resumed
    by the event loop until it completes, at which point awaiting the task yields the coroutine's
    return value.
    """

    def __iter__(self) -> 'Task[_T]':
        """Await this task until it completes.

        When the task finishes, awaiting it produces the coroutine's return value.
        """
        ...


class EventLoop:
    """A cooperative event loop for Venice coroutines.

    The event loop maintains a queue of ready tasks and a queue of sleeping tasks. Calling `run`
    repeatedly advances scheduled coroutines until there is no more work left to do.
    """

    def __init__(self) -> None:
        """Create a new empty event loop."""
        ...

    def spawn(self, coro: Awaitable[_T]) -> Task[_T]:
        """Schedule a coroutine on this event loop and return its task handle."""
        ...

    def run(self) -> None:
        """Run this event loop until there are no ready or sleeping tasks remaining."""
        ...


class Sleep:
    """An awaitable sleep operation.

    Instances of this class yield control back to the Venice event loop until the requested
    duration has elapsed.
    """

    def __init__(self, interval: float, unit: TimeUnit) -> None:
        """Create a sleep operation for the given duration and time unit."""
        ...

    def __iter__(self) -> 'Sleep':
        """Await this sleep operation until it completes."""
        ...


def get_running_loop() -> EventLoop | None:
    """Return the currently running event loop.

    If no event loop is currently running, this returns `None`.
    """
    ...


def run(coro: Awaitable[Any]) -> None:
    """Create a fresh event loop, schedule `coro`, and run it to completion.

    This function installs the new loop as the running loop for the duration of the call.
    """
    ...


def spawn(coro: Awaitable[_T]) -> Task[_T]:
    """Schedule `coro` on the currently running event loop.

    Raises a runtime error if called when no event loop is running.
    """
    ...


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
