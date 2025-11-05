pub mod direction;
pub mod gearset;

use micropython_rs::{
    except::raise_value_error,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::motor::Motor;

use crate::{
    args::{ArgType, ArgValue, Args},
    modvenice::motor::{direction::DirectionObj, gearset::GearsetObj},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::{
        RegistryGuard,
        registries::{self, PortNumber},
    },
};

#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, Motor>,
}

static MOTOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Motor)).set_slot_make_new(motor_make_new);

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

    let guard = registries::try_lock_port(port, |port| Motor::new(port, gearset, direction))
        .unwrap_or_else(|_| panic!("port is already in use"));

    alloc_obj(MotorObj {
        base: ObjBase::new::<MotorObj>(),
        guard,
    })
}
