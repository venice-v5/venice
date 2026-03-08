pub mod gesture;
pub mod rgb;

use argparse::Args;
use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::optical::OpticalSensor;

use crate::{
    devices::{self, PortNumber},
    fun::{fun1, fun2, fun3},
    modvenice::{
        optical::{
            gesture::GestureObj,
            rgb::{OpticalRawObj, OpticalRgbObj},
        },
        raise_port_error,
        units::time::TimeUnitObj,
    },
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct OpticalSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, OpticalSensor>,
}

pub static OPTICAL_SENSOR_OBJ_TYPE: ObjFullType = ObjFullType::new(
    TypeFlags::empty(),
    qstr!(OpticalSensor),
)
.set_make_new(make_new_from_fn!(optical_make_new))
.set_locals_dict(const_dict![
    qstr!(MIN_INTEGRATION_TIME_MS) => Obj::from_float(OpticalSensor::MIN_INTEGRATION_TIME.as_millis() as f32),
    qstr!(MAX_INTEGRATION_TIME_MS) => Obj::from_float(OpticalSensor::MAX_INTEGRATION_TIME.as_millis() as f32),
    qstr!(GESTURE_UPDATE_INTERVAL_MS) => Obj::from_float(OpticalSensor::GESTURE_UPDATE_INTERVAL.as_millis() as f32),

    qstr!(get_hue) => Obj::from_static(&fun1!(optical_get_hue, &OpticalSensorObj)),
    qstr!(get_saturation) => Obj::from_static(&fun1!(optical_get_saturation, &OpticalSensorObj)),
    qstr!(get_brightness) => Obj::from_static(&fun1!(optical_get_brightness, &OpticalSensorObj)),
    qstr!(get_proximity) => Obj::from_static(&fun1!(optical_get_proximity, &OpticalSensorObj)),
    qstr!(get_color) => Obj::from_static(&fun1!(optical_get_color, &OpticalSensorObj)),
    qstr!(get_raw_color) => Obj::from_static(&fun1!(optical_get_raw_color, &OpticalSensorObj)),
    qstr!(get_last_gesture) => Obj::from_static(&fun1!(optical_get_last_gesture, &OpticalSensorObj)),
    qstr!(get_led_brightness) => Obj::from_static(&fun1!(optical_get_led_brightness, &OpticalSensorObj)),
    qstr!(set_led_brightness) => Obj::from_static(&fun2!(optical_set_led_brightness, &OpticalSensorObj, f32)),
    qstr!(get_integation_time) => Obj::from_static(&fun2!(optical_get_integration_time, &OpticalSensorObj, &TimeUnitObj)),
    qstr!(set_integation_time) => Obj::from_static(&fun3!(optical_set_integration_time, &OpticalSensorObj, f32, &TimeUnitObj)),
    qstr!(get_status) => Obj::from_static(&fun1!(optical_get_status, &OpticalSensorObj)),

    qstr!(free) => Obj::from_static(&fun1!(optical_free, &OpticalSensorObj)),
]);

unsafe impl ObjTrait for OpticalSensorObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = OPTICAL_SENSOR_OBJ_TYPE.as_obj_type();
}

fn optical_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let mut reader = Args::new(n_pos, n_kw, args).reader(token());
    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token(), c"port number must be between 1 and 21"));

    alloc_obj(OpticalSensorObj {
        base: ObjBase::new(ty),
        guard: devices::lock_port(port, |p| OpticalSensor::new(p)),
    })
}

fn optical_get_hue(this: &OpticalSensorObj) -> Obj {
    Obj::from_float(
        this.guard
            .borrow()
            .hue()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32,
    )
}

fn optical_get_saturation(this: &OpticalSensorObj) -> Obj {
    Obj::from_float(
        this.guard
            .borrow()
            .saturation()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32,
    )
}

fn optical_get_brightness(this: &OpticalSensorObj) -> Obj {
    Obj::from_float(
        this.guard
            .borrow()
            .brightness()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32,
    )
}

fn optical_get_proximity(this: &OpticalSensorObj) -> Obj {
    Obj::from_float(
        this.guard
            .borrow()
            .proximity()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32,
    )
}

fn optical_get_color(this: &OpticalSensorObj) -> Obj {
    alloc_obj(OpticalRgbObj::new(
        this.guard
            .borrow()
            .color()
            .unwrap_or_else(|e| raise_port_error!(e)),
    ))
}

fn optical_get_raw_color(this: &OpticalSensorObj) -> Obj {
    alloc_obj(OpticalRawObj::new(
        this.guard
            .borrow()
            .raw_color()
            .unwrap_or_else(|e| raise_port_error!(e)),
    ))
}

fn optical_get_last_gesture(this: &OpticalSensorObj) -> Obj {
    this.guard
        .borrow()
        .last_gesture()
        .unwrap_or_else(|e| raise_port_error!(e))
        .map(|g| alloc_obj(GestureObj::new(g)))
        .unwrap_or(Obj::NONE)
}

fn optical_get_led_brightness(this: &OpticalSensorObj) -> Obj {
    Obj::from_float(
        this.guard
            .borrow()
            .led_brightness()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32,
    )
}

fn optical_set_led_brightness(this: &OpticalSensorObj, brightness: f32) -> Obj {
    this.guard
        .borrow_mut()
        .set_led_brightness(brightness as f64)
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn optical_get_integration_time(this: &OpticalSensorObj, unit: &TimeUnitObj) -> Obj {
    Obj::from_float(
        unit.unit().dur_to_float(
            this.guard
                .borrow()
                .integration_time()
                .unwrap_or_else(|e| raise_port_error!(e)),
        ),
    )
}

fn optical_set_integration_time(this: &OpticalSensorObj, time: f32, unit: &TimeUnitObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_integration_time(unit.unit().float_to_dur(time))
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn optical_get_status(this: &OpticalSensorObj) -> Obj {
    Obj::from_int(
        this.guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e)) as i32, // should be OK to cast, the bits themselves stay the same
    )
}

fn optical_free(this: &OpticalSensorObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
