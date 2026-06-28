"""
Robot battery information.

This module provides APIs for retrieving information about the robot's battery and state of
charge.
"""
from __future__ import annotations

def get_capacity() -> float:
    """
    Returns the charge capacity of the robot's battery in the range of [0.0, 1.0].

    A value of `0.0` indicates a completely empty battery, while a value of `1.0` indicates a
    fully-charged battery.

    # Examples

    ```python
    from venice import *

    capacity = battery.get_capacity()
    print(f"Battery at {capacity:.0%} capacity")

    if capacity < 0.2:
    print("Warning: Low battery!")
    ```
    """
    ...

def get_current() -> float:
    """
    Returns the electric current of the robot's battery in amps.

    Maximum current draw on the V5 battery is 20 Amps.

    # Examples

    ```python
    from venice import *

    current = battery.get_current()

    print(f"Drawing {current} amps")
    ```
    """
    ...

def get_temperature() -> int:
    """
    Returns the internal temperature of the robot's battery in degrees Celsius.

    # Examples

    ```python
    from venice import *

    temp = battery.get_temperature()
    print(f"Battery temperature: {temp}°C")

    # Check if battery is too hot
    if temp > 45:
    print("Warning: Battery temperature critical!")
    ```
    """
    ...

def get_voltage() -> float:
    """
    Returns the robot's battery voltage in volts.

    Nominal battery voltage on the V5 brain is 12.8V.

    # Examples

    ```python
    from venice import *

    voltage = battery.get_voltage()
    print("Battery voltage: {voltage} V")
    ```
    """
    ...
