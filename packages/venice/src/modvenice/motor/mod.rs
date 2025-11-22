pub mod brake;
pub mod direction;
pub mod gearset;
pub mod motor_type;

use brake::BrakeModeObj;
use direction::DirectionObj;
use gearset::GearsetObj;
use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::{math::Direction, smart::motor::Motor};

use super::raise_device_error;
use crate::{
    args::Args,
    devices::{self, PortNumber},
    fun::{fun_var_from_fn, fun1_from_fn, fun2_from_fn, fun3_from_fn},
    modvenice::{motor::motor_type::MotorTypeObj, units::rotation::RotationUnitObj},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct MotorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, Motor>,
}

pub(crate) static ABSTRACT_MOTOR_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(AbstractMotor))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(WRITE_INTERVAL_MS) => Obj::from_int(5),

        qstr!(set_voltage) => Obj::from_static(&fun2_from_fn!(motor_set_voltage,&MotorObj, f32)),
        qstr!(set_velocity) => Obj::from_static(&fun2_from_fn!(motor_set_velocity,&MotorObj, i32)),
        qstr!(brake) => Obj::from_static(&fun2_from_fn!(motor_brake,&MotorObj, &BrakeModeObj)),
        qstr!(set_position_target) => Obj::from_static(&fun_var_from_fn!(motor_set_position_target)),
        qstr!(is_exp) => Obj::from_static(&fun1_from_fn!(motor_is_exp, &MotorObj)),
        qstr!(is_v5) => Obj::from_static(&fun1_from_fn!(motor_is_v5, &MotorObj)),
        qstr!(max_voltage) => Obj::from_static(&fun1_from_fn!(motor_max_voltage, &MotorObj)),
        qstr!(velocity) => Obj::from_static(&fun1_from_fn!(motor_velocity, &MotorObj)),
        qstr!(power) => Obj::from_static(&fun1_from_fn!(motor_power, &MotorObj)),
        qstr!(torque) => Obj::from_static(&fun1_from_fn!(motor_torque, &MotorObj)),
        qstr!(voltage) => Obj::from_static(&fun1_from_fn!(motor_voltage, &MotorObj)),
        qstr!(raw_position) => Obj::from_static(&fun1_from_fn!(motor_raw_position, &MotorObj)),
        qstr!(current) => Obj::from_static(&fun1_from_fn!(motor_current, &MotorObj)),
        qstr!(efficiency) => Obj::from_static(&fun1_from_fn!(motor_efficiency, &MotorObj)),
        qstr!(current_limit) => Obj::from_static(&fun1_from_fn!(motor_current_limit, &MotorObj)),
        qstr!(voltage_limit) => Obj::from_static(&fun1_from_fn!(motor_voltage_limit, &MotorObj)),
        qstr!(temperature) => Obj::from_static(&fun1_from_fn!(motor_temperature, &MotorObj)),
        qstr!(set_profiled_velocity) => Obj::from_static(&fun2_from_fn!(motor_set_profiled_velocity, &MotorObj, i32)),
        qstr!(reset_position) => Obj::from_static(&fun1_from_fn!(motor_reset_position, &MotorObj)),
        qstr!(set_current_limit) => Obj::from_static(&fun2_from_fn!(motor_set_current_limit, &MotorObj, f32)),
        qstr!(set_voltage_limit) => Obj::from_static(&fun2_from_fn!(motor_set_voltage_limit, &MotorObj, f32)),
        qstr!(is_over_temperature) => Obj::from_static(&fun1_from_fn!(motor_is_over_temperature, &MotorObj)),
        qstr!(is_over_current) => Obj::from_static(&fun1_from_fn!(motor_is_over_current, &MotorObj)),
        qstr!(is_driver_fault) => Obj::from_static(&fun1_from_fn!(motor_is_driver_fault, &MotorObj)),
        qstr!(is_driver_over_current) => Obj::from_static(&fun1_from_fn!(motor_is_driver_over_current, &MotorObj)),
        qstr!(status) => Obj::from_static(&fun1_from_fn!(motor_status, &MotorObj)),
        qstr!(faults) => Obj::from_static(&fun1_from_fn!(motor_faults, &MotorObj)),
        qstr!(motor_type) => Obj::from_static(&fun1_from_fn!(motor_motor_type, &MotorObj)),
        qstr!(position) => Obj::from_static(&fun2_from_fn!(motor_position, &MotorObj, &RotationUnitObj)),
        qstr!(set_position) => Obj::from_static(&fun3_from_fn!(motor_set_position, &MotorObj, f32, &RotationUnitObj)),
        qstr!(set_direction) => Obj::from_static(&fun2_from_fn!(motor_set_direction, &MotorObj, &DirectionObj)),
        qstr!(direction) => Obj::from_static(&fun1_from_fn!(motor_direction, &MotorObj)),
    ]);

pub(crate) static MOTOR_V5_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(MotorV5))
    .set_make_new(make_new_from_fn!(motor_v5_make_new))
    .set_slot_parent(ABSTRACT_MOTOR_OBJ_TYPE.as_obj_type())
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(MAX_VOLTAGE) => Obj::from_float(12.0),
        qstr!(set_gearset) => Obj::from_static(&fun2_from_fn!(motor_set_gearset,&MotorObj, &GearsetObj)),
        qstr!(gearset) => Obj::from_static(&fun1_from_fn!(motor_gearset,&MotorObj)),
    ]);

pub(crate) static MOTOR_EXP_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(MotorExp))
        .set_make_new(make_new_from_fn!(motor_exp_make_new))
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(MAX_VOLTAGE) => Obj::from_float(8.0),
        ]);

unsafe impl ObjTrait for MotorObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = ABSTRACT_MOTOR_OBJ_TYPE.as_obj_type();
}

fn motor_v5_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token().unwrap();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(3, 3);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let direction: &DirectionObj = reader.next_positional();

    let gearset: &GearsetObj = reader.next_positional();

    let guard = devices::try_lock_port(port, |port| {
        Motor::new(port, gearset.gearset(), direction.direction())
    })
    .unwrap_or_else(|_| panic!("port is already in use"));

    if guard.borrow().is_exp() {
        raise_device_error(token, "Invalid motor type, expected V5, found Exp")
    }

    alloc_obj(MotorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn motor_exp_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token().unwrap();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(2, 2);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let direction: &DirectionObj = reader.next_positional();

    let guard = devices::try_lock_port(port, |port| Motor::new_exp(port, direction.direction()))
        .unwrap_or_else(|_| panic!("port is already in use"));

    if guard.borrow().is_v5() {
        raise_device_error(token, "Invalid motor type, expected Exp, found V5")
    }

    alloc_obj(MotorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn motor_set_voltage(this: &MotorObj, volts: f32) -> Obj {
    this.guard
        .borrow_mut()
        .set_voltage(volts as f64)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_set_velocity(this: &MotorObj, rpm: i32) -> Obj {
    this.guard
        .borrow_mut()
        .set_velocity(rpm)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));

    Obj::NONE
}

fn motor_brake(this: &MotorObj, mode: &BrakeModeObj) -> Obj {
    this.guard
        .borrow_mut()
        .brake(mode.mode())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_set_gearset(this: &MotorObj, gearset: &GearsetObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_gearset(gearset.gearset())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_is_exp(this: &MotorObj) -> Obj {
    Obj::from_bool(this.guard.borrow().is_exp())
}

fn motor_is_v5(this: &MotorObj) -> Obj {
    Obj::from_bool(this.guard.borrow().is_v5())
}

fn motor_max_voltage(this: &MotorObj) -> Obj {
    Obj::from_float(this.guard.borrow().max_voltage() as f32)
}

fn motor_gearset(this: &MotorObj) -> Obj {
    let gearset = this
        .guard
        .borrow()
        .gearset()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_static(GearsetObj::new_static(gearset))
}

fn motor_set_position_target(args: &[Obj]) -> Obj {
    let mut reader = Args::new(args.len(), 0, args).reader(token().unwrap());
    // self, position, position units, velocity
    reader.assert_npos(4, 4);
    let motor = reader.next_positional::<&MotorObj>();

    let position_val = reader.next_positional();

    let unit_obj = reader.next_positional::<&RotationUnitObj>();

    let velocity_val = reader.next_positional();
    let angle = unit_obj.unit().from_float(position_val);

    motor
        .guard
        .borrow_mut()
        .set_position_target(angle, velocity_val)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));

    Obj::NONE
}

fn motor_velocity(this: &MotorObj) -> Obj {
    let vel = this
        .guard
        .borrow()
        .velocity()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(vel as f32)
}

fn motor_power(this: &MotorObj) -> Obj {
    let pwr = this
        .guard
        .borrow()
        .power()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(pwr as f32)
}

fn motor_torque(this: &MotorObj) -> Obj {
    let trq = this
        .guard
        .borrow()
        .torque()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(trq as f32)
}

fn motor_voltage(this: &MotorObj) -> Obj {
    let volt = this
        .guard
        .borrow()
        .voltage()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(volt as f32)
}

fn motor_raw_position(this: &MotorObj) -> Obj {
    let pos = this
        .guard
        .borrow()
        .raw_position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(pos)
}

fn motor_current(this: &MotorObj) -> Obj {
    let curr = this
        .guard
        .borrow()
        .current()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(curr as f32)
}

fn motor_efficiency(this: &MotorObj) -> Obj {
    let eff = this
        .guard
        .borrow()
        .efficiency()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(eff as f32)
}

fn motor_current_limit(this: &MotorObj) -> Obj {
    let lim = this
        .guard
        .borrow()
        .current_limit()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(lim as f32)
}

fn motor_voltage_limit(this: &MotorObj) -> Obj {
    let lim = this
        .guard
        .borrow()
        .voltage_limit()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(lim as f32)
}

fn motor_temperature(this: &MotorObj) -> Obj {
    let temp = this
        .guard
        .borrow()
        .temperature()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(temp as f32)
}

fn motor_set_profiled_velocity(this: &MotorObj, velocity: i32) -> Obj {
    this.guard
        .borrow_mut()
        .set_profiled_velocity(velocity)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_reset_position(this: &MotorObj) -> Obj {
    this.guard
        .borrow_mut()
        .reset_position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_set_current_limit(this: &MotorObj, limit: f32) -> Obj {
    this.guard
        .borrow_mut()
        .set_current_limit(limit as f64)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_set_voltage_limit(this: &MotorObj, limit: f32) -> Obj {
    this.guard
        .borrow_mut()
        .set_voltage_limit(limit as f64)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_is_over_temperature(this: &MotorObj) -> Obj {
    let is_over = this
        .guard
        .borrow()
        .is_over_temperature()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}

fn motor_is_over_current(this: &MotorObj) -> Obj {
    let is_over = this
        .guard
        .borrow()
        .is_over_current()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}

fn motor_is_driver_fault(this: &MotorObj) -> Obj {
    let is_fault = this
        .guard
        .borrow()
        .is_driver_fault()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_fault)
}

fn motor_is_driver_over_current(this: &MotorObj) -> Obj {
    let is_over = this
        .guard
        .borrow()
        .is_driver_over_current()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}

fn motor_motor_type(this: &MotorObj) -> Obj {
    let mt = this.guard.borrow().motor_type();
    Obj::from_static(MotorTypeObj::new_static(mt))
}

fn motor_position(this: &MotorObj, unit: &RotationUnitObj) -> Obj {
    let angle = this
        .guard
        .borrow()
        .position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(unit.unit().in_angle(angle))
}

fn motor_set_position(this: &MotorObj, position: f32, unit: &RotationUnitObj) -> Obj {
    let angle = unit.unit().from_float(position);
    this.guard
        .borrow_mut()
        .set_position(angle)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_set_direction(this: &MotorObj, direction: &DirectionObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_direction(direction.direction())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn motor_direction(this: &MotorObj) -> Obj {
    let dir = this
        .guard
        .borrow()
        .direction()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    match dir {
        Direction::Forward => Obj::from_static(&DirectionObj::FORWARD),
        Direction::Reverse => Obj::from_static(&DirectionObj::REVERSE),
    }
}

fn motor_status(this: &MotorObj) -> Obj {
    let status = this
        .guard
        .borrow()
        .status()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(status.bits() as i32)
}

fn motor_faults(this: &MotorObj) -> Obj {
    let faults = this
        .guard
        .borrow()
        .faults()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(faults.bits() as i32)
}
