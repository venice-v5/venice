use micropython_macros::fun;
use micropython_rs::{const_dict, map::Dict, obj::Obj};

/// Returns the charge capacity of the robot's battery in the range of [0.0, 1.0].
///
/// A value of `0.0` indicates a completely empty battery, while a value of `1.0` indicates a
/// fully-charged battery.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// capacity = battery.get_capacity()
/// print(f"Battery at {capacity:.0%} capacity")
///
/// if capacity < 0.2:
///     print("Warning: Low battery!")
/// ```
#[fun]
#[stub(sig = "() -> float")]
fn get_capacity() -> Obj {
    (vexide_devices::battery::capacity() as f32).into()
}

/// Returns the electric current of the robot's battery in amps.
///
/// Maximum current draw on the V5 battery is 20 Amps.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// current = battery.get_current()
///
/// print(f"Drawing {current} amps")
/// ```
#[fun]
#[stub(sig = "() -> float")]
fn get_current() -> Obj {
    (vexide_devices::battery::current() as f32).into()
}

/// Returns the internal temperature of the robot's battery in degrees Celsius.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// temp = battery.get_temperature()
/// print(f"Battery temperature: {temp}°C")
///
/// # Check if battery is too hot
/// if temp > 45:
///     print("Warning: Battery temperature critical!")
/// ```
#[fun]
#[stub(sig = "() -> int")]
fn get_temperature() -> Obj {
    (vexide_devices::battery::temperature() as i32).into()
}

/// Returns the robot's battery voltage in volts.
///
/// Nominal battery voltage on the V5 brain is 12.8V.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// voltage = battery.get_voltage()
/// print("Battery voltage: {voltage} V")
/// ```
#[fun]
#[stub(sig = "() -> float")]
fn get_voltage() -> Obj {
    (vexide_devices::battery::voltage() as f32).into()
}

pub const BATTERY_DICT: &Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(battery)),
    qstr!(get_capacity) => get_capacity_obj,
    qstr!(get_current) => get_current_obj,
    qstr!(get_temperature) => get_temperature_obj,
    qstr!(get_voltage) => get_voltage_obj,
];
