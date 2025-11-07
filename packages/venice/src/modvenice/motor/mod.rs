pub mod brake;
pub mod direction;
pub mod gearset;
pub mod motor_type;

use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    fun::{Fun1, Fun2, FunVar},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::motor::{Motor, MotorStatus};

use crate::{
    args::{ArgType, ArgValue, Args, ArgsReader},
    devices::{self, PortNumber},
    modvenice::raise_device_error,
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

use {
    brake::BrakeModeObj, direction::DirectionObj, gearset::GearsetObj
};

use crate::modvenice::units::rotation::RotationUnitObj;

#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, Motor>,
}

static MOTOR_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Motor))
    .set_slot_make_new(motor_make_new)
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(set_voltage) => Obj::from_static(&Fun2::new(motor_set_voltage)),
        qstr!(set_velocity) => Obj::from_static(&Fun2::new(motor_set_velocity)),
        qstr!(brake) => Obj::from_static(&Fun2::new(motor_brake)),
        qstr!(set_gearset) => Obj::from_static(&Fun2::new(motor_set_gearset)),
        qstr!(gearset) => Obj::from_static(&Fun1::new(motor_gearset)),
        qstr!(set_position_target) => Obj::from_static(&FunVar::new(motor_set_position_target)),
        qstr!(is_exp) => Obj::from_static(&Fun1::new(motor_is_exp)),
        qstr!(is_v5) => Obj::from_static(&Fun1::new(motor_is_v5)),
        qstr!(max_voltage) => Obj::from_static(&Fun1::new(motor_max_voltage)),
        qstr!(velocity) => Obj::from_static(&Fun1::new(motor_velocity)),
        qstr!(power) => Obj::from_static(&Fun1::new(motor_power)),
        qstr!(torque) => Obj::from_static(&Fun1::new(motor_torque)),
        qstr!(voltage) => Obj::from_static(&Fun1::new(motor_voltage)),
        qstr!(raw_position) => Obj::from_static(&Fun1::new(motor_raw_position)),
        qstr!(current) => Obj::from_static(&Fun1::new(motor_current)),
        qstr!(efficiency) => Obj::from_static(&Fun1::new(motor_efficiency)),
        qstr!(current_limit) => Obj::from_static(&Fun1::new(motor_current_limit)),
        qstr!(voltage_limit) => Obj::from_static(&Fun1::new(motor_voltage_limit)),
        qstr!(temperature) => Obj::from_static(&Fun1::new(motor_temperature)),
        qstr!(set_profiled_velocity) => Obj::from_static(&Fun2::new(motor_set_profiled_velocity)),
        qstr!(reset_position) => Obj::from_static(&Fun1::new(motor_reset_position)),
        qstr!(set_current_limit) => Obj::from_static(&Fun2::new(motor_set_current_limit)),
        qstr!(set_voltage_limit) => Obj::from_static(&Fun2::new(motor_set_voltage_limit)),
        qstr!(is_over_temperature) => Obj::from_static(&Fun1::new(motor_is_over_temperature)),
        qstr!(is_over_current) => Obj::from_static(&Fun1::new(motor_is_over_current)),
        qstr!(is_driver_fault) => Obj::from_static(&Fun1::new(motor_is_driver_fault)),
        qstr!(is_driver_over_current) => Obj::from_static(&Fun1::new(motor_is_driver_over_current)),
    ]);

unsafe impl ObjTrait for MotorObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = MOTOR_OBJ_TYPE.as_obj_type();
}

extern "C" fn motor_make_new(
    _: *const ObjType,
    n_pos: usize,
    n_kw: usize,
    arg_ptr: *const Obj,
) -> Obj {
    let token = token().unwrap();

    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, arg_ptr) }.reader(token);
    args.assert_npos(2, 4).assert_nkw(0, 0);
    let port = PortNumber::from_i32(args.next_positional(ArgType::Int).as_int())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let direction = args
        .next_positional_or(
            ArgType::Obj(DirectionObj::OBJ_TYPE),
            ArgValue::Obj(Obj::from_static(&DirectionObj::FORWARD)),
        )
        .as_obj()
        .try_to_obj::<DirectionObj>()
        .unwrap()
        .direction();

    let exp = args
        .get_kw_or(b"exp", ArgType::Bool, ArgValue::Bool(false))
        .as_bool();

    let guard = devices::try_lock_port(port, |port| {
        if exp {
            Motor::new_exp(port, direction)
        } else {
            let gearset = args
                .next_positional(ArgType::Obj(GearsetObj::OBJ_TYPE))
                .as_obj()
                .try_to_obj::<GearsetObj>()
                .unwrap()
                .gearset();
            Motor::new(port, gearset, direction)
        }
    })
    .unwrap_or_else(|_| panic!("port is already in use"));

    alloc_obj(MotorObj {
        base: ObjBase::new(MotorObj::OBJ_TYPE),
        guard,
    })
}

extern "C" fn motor_set_voltage(self_in: Obj, volts: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .set_voltage(volts.try_to_float().unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <float> for argument #1, found <{}>",
                    ArgType::of(&volts)
                ),
            )
        }) as f64)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));

    Obj::NONE
}

extern "C" fn motor_set_velocity(self_in: Obj, rpm: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .set_velocity(rpm.try_to_int().unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <int> for argument #1, found <{}>",
                    ArgType::of(&rpm)
                ),
            )
        }))
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));

    Obj::NONE
}

extern "C" fn motor_brake(self_in: Obj, mode: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let mode = mode
        .try_to_obj::<BrakeModeObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <BrakeMode> for argument #1, found <{}>",
                    ArgType::of(&mode)
                ),
            )
        })
        .mode();
    motor
        .guard
        .borrow_mut()
        .brake(mode)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_set_gearset(self_in: Obj, gearset: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let gearset = gearset
        .try_to_obj::<GearsetObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <Gearset> for argument #1, found <{}>",
                    ArgType::of(&gearset)
                ),
            )
        })
        .gearset();
    motor
        .guard
        .borrow_mut()
        .set_gearset(gearset)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_is_exp(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_exp = motor.guard.borrow().is_exp();
    Obj::from_bool(is_exp)
}

extern "C" fn motor_is_v5(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_v5 = motor.guard.borrow().is_v5();
    Obj::from_bool(is_v5)
}

extern "C" fn motor_max_voltage(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let max_volts = motor.guard.borrow().max_voltage();
    Obj::from_float(max_volts as f32)
}

extern "C" fn motor_gearset(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let gearset = motor
        .guard
        .borrow()
        .gearset()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_static(GearsetObj::new_static(gearset))
}

extern "C" fn motor_set_position_target(n_args: usize, ptr: *const Obj) -> Obj {
    let token = token().unwrap();
    let mut reader = unsafe { Args::from_ptr(n_args, 0, ptr) }.reader(token);
    // self, position, position units, velocity
    reader.assert_npos(4, 4);
    let motor = reader.next_positional(ArgType::Obj(MotorObj::OBJ_TYPE)).as_obj();
    let motor = motor.try_to_obj::<MotorObj>().unwrap();

    let position_val = reader.next_positional(ArgType::Float).as_float();

    let unit_obj = reader.next_positional(ArgType::Obj(RotationUnitObj::OBJ_TYPE)).as_obj();
    let unit_obj = unit_obj.try_to_obj::<RotationUnitObj>().unwrap();

    let velocity_val = reader.next_positional(ArgType::Int).as_int();
    let angle = unit_obj.unit().from_float(position_val);

    motor
        .guard
        .borrow_mut()
        .set_position_target(angle, velocity_val)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));

    Obj::NONE
}

extern "C" fn motor_velocity(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let vel = motor.guard.borrow().velocity().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(vel as f32)
}

extern "C" fn motor_power(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let pwr = motor.guard.borrow().power().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(pwr as f32)
}

extern "C" fn motor_torque(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let trq = motor.guard.borrow().torque().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(trq as f32)
}

extern "C" fn motor_voltage(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let volt = motor.guard.borrow().voltage().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(volt as f32)
}

extern "C" fn motor_raw_position(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let pos = motor.guard.borrow().raw_position().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(pos)
}

extern "C" fn motor_current(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let curr = motor.guard.borrow().current().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(curr as f32)
}

extern "C" fn motor_efficiency(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let eff = motor.guard.borrow().efficiency().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(eff as f32)
}

extern "C" fn motor_current_limit(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let lim = motor.guard.borrow().current_limit().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(lim as f32)
}

extern "C" fn motor_voltage_limit(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let lim = motor.guard.borrow().voltage_limit().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(lim as f32)
}

extern "C" fn motor_temperature(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let temp = motor.guard.borrow().temperature().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(temp as f32)
}

extern "C" fn motor_set_profiled_velocity(self_in: Obj, velocity: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .set_profiled_velocity(velocity.try_to_int().unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <int> for argument #1, found <{}>",
                    ArgType::of(&velocity)
                ),
            )
        }))
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_reset_position(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .reset_position()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_set_current_limit(self_in: Obj, limit: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .set_current_limit(limit.try_to_float().unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <float> for argument #1, found <{}>",
                    ArgType::of(&limit)
                ),
            )
        }) as f64)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_set_voltage_limit(self_in: Obj, limit: Obj) -> Obj {
    let token = token().unwrap();
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    motor
        .guard
        .borrow_mut()
        .set_voltage_limit(limit.try_to_float().unwrap_or_else(|| {
            raise_type_error(
                token,
                format!(
                    "expected <float> for argument #1, found <{}>",
                    ArgType::of(&limit)
                ),
            )
        }) as f64)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));
    Obj::NONE
}

extern "C" fn motor_is_over_temperature(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_over = motor.guard.borrow().is_over_temperature().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}

extern "C" fn motor_is_over_current(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_over = motor.guard.borrow().is_over_current().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}

extern "C" fn motor_is_driver_fault(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_fault = motor.guard.borrow().is_driver_fault().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_fault)
}

extern "C" fn motor_is_driver_over_current(self_in: Obj) -> Obj {
    let motor = self_in.try_to_obj::<MotorObj>().unwrap();
    let is_over = motor.guard.borrow().is_driver_over_current().unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_bool(is_over)
}
