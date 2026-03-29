use micropython_rs::{const_dict, fun::Fun0, map::Dict, obj::Obj};

extern "C" fn get_capacity() -> Obj {
    (vexide_devices::battery::capacity() as f32).into()
}

extern "C" fn get_current() -> Obj {
    (vexide_devices::battery::current() as f32).into()
}

extern "C" fn get_temperature() -> Obj {
    (vexide_devices::battery::temperature() as i32).into()
}

extern "C" fn get_voltage() -> Obj {
    (vexide_devices::battery::voltage() as f32).into()
}

pub const BATTERY_DICT: &Dict = const_dict![
    qstr!(get_capacity) => Obj::from_static(&Fun0::new(get_capacity)),
    qstr!(get_current) => Obj::from_static(&Fun0::new(get_current)),
    qstr!(get_temperature) => Obj::from_static(&Fun0::new(get_temperature)),
    qstr!(get_voltage) => Obj::from_static(&Fun0::new(get_voltage)),
];
