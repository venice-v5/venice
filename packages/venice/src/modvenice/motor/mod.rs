pub mod direction;
pub mod gearset;

use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    fun::Fun2,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::motor::Motor;

use crate::{
    args::{ArgType, ArgValue, Args},
    devices::{self, PortNumber},
    modvenice::{
        motor::{direction::DirectionObj, gearset::GearsetObj},
        raise_device_error,
    },
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, Motor>,
}

static MOTOR_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Motor))
    .set_slot_make_new(motor_make_new)
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(set_voltage) => Obj::from_static(&Fun2::new(motor_set_voltage))
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

    let gearset = args
        .next_positional(ArgType::Obj(GearsetObj::OBJ_TYPE))
        .as_obj()
        .try_to_obj::<GearsetObj>()
        .unwrap()
        .gearset();

    let direction = args
        .next_positional_or(
            ArgType::Obj(DirectionObj::OBJ_TYPE),
            ArgValue::Obj(Obj::from_static(&DirectionObj::FORWARD)),
        )
        .as_obj()
        .try_to_obj::<DirectionObj>()
        .unwrap()
        .direction();

    let guard = devices::try_lock_port(port, |port| Motor::new(port, gearset, direction))
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
                    "expected float for argument #1, found {}",
                    ArgType::of(&volts)
                ),
            )
        }) as f64)
        .unwrap_or_else(|e| raise_device_error(token, format!("{e}")));

    Obj::NONE
}
