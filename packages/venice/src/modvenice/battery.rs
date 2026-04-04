use micropython_macros::fun;
use micropython_rs::{const_dict, map::Dict, obj::Obj};

#[fun]
fn get_capacity() -> Obj {
    (vexide_devices::battery::capacity() as f32).into()
}

#[fun]
fn get_current() -> Obj {
    (vexide_devices::battery::current() as f32).into()
}

#[fun]
fn get_temperature() -> Obj {
    (vexide_devices::battery::temperature() as i32).into()
}

#[fun]
fn get_voltage() -> Obj {
    (vexide_devices::battery::voltage() as f32).into()
}

pub const BATTERY_DICT: &Dict = const_dict![
    qstr!(get_capacity) => get_capacity_obj,
    qstr!(get_current) => get_current_obj,
    qstr!(get_temperature) => get_temperature_obj,
    qstr!(get_voltage) => get_voltage_obj,
];
