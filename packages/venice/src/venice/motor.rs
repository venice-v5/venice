use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::{
    Motor,
    motor::{Direction, Gearset},
};

use crate::{
    obj::alloc_obj,
    qstrgen::qstr,
    venice::registries::{self, PortNumber},
};

#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    port: PortNumber,
}

pub static MOTOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Motor)).set_slot_make_new(motor_make_new);

const GEARSET_RED: i32 = 0;
const GEARSET_GREEN: i32 = 1;
const GEARSET_BLUE: i32 = 2;

const DIRECTION_FORWARD: i32 = 0;
const DIRECTION_REVERSE: i32 = 1;

pub fn gearset_from_obj(obj: Obj) -> Option<Gearset> {
    obj.as_small_int().and_then(|int| match int {
        GEARSET_RED => Some(Gearset::Red),
        GEARSET_GREEN => Some(Gearset::Green),
        GEARSET_BLUE => Some(Gearset::Blue),
        _ => None,
    })
}

pub fn direction_from_obj(obj: Obj) -> Option<Direction> {
    obj.as_small_int().and_then(|int| match int {
        DIRECTION_FORWARD => Some(Direction::Forward),
        DIRECTION_REVERSE => Some(Direction::Reverse),
        _ => None,
    })
}

pub static GEARSET_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(RED) => Obj::from_small_int(GEARSET_RED),
        qstr!(GREEN) => Obj::from_small_int(GEARSET_GREEN),
        qstr!(BLUE) => Obj::from_small_int(GEARSET_BLUE),
    ]);

pub static DIRECTION_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Direction))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(FORWARD) => Obj::from_small_int(DIRECTION_FORWARD),
        qstr!(REVERSE) => Obj::from_small_int(DIRECTION_REVERSE),
    ]);

unsafe impl ObjTrait for MotorObj {
    const OBJ_TYPE: *const micropython_rs::obj::ObjType = MOTOR_OBJ_TYPE.as_obj_type_ptr();
}

extern "C" fn motor_make_new(
    _: *const ObjType,
    n_args: usize,
    n_kw: usize,
    args: *const Obj,
) -> Obj {
    let token = token().unwrap();
    if n_kw != 0 {
        raise_type_error(token, "function does not accept keyword arguments");
    }

    if n_args < 2 || n_args > 4 {
        raise_type_error(token, "function accepts at least 2 arguments and at most 4");
    }

    let args = unsafe { std::slice::from_raw_parts(args, n_args) };
    let port = PortNumber::from_i32(
        args[0]
            .as_small_int()
            .unwrap_or_else(|| raise_type_error(token, "expected integer for port number")),
    )
    .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let gearset = gearset_from_obj(args[1])
        .unwrap_or_else(|| raise_type_error(token, "expect Gearset object"));

    let direction = match args.get(2) {
        Some(d) => direction_from_obj(*d)
            .unwrap_or_else(|| raise_type_error(token, "expected Direction object")),
        None => Direction::Reverse,
    };

    let _exp = match args.get(3) {
        Some(e) => e
            .as_bool()
            .unwrap_or_else(|| raise_type_error(token, "expected bool")),
        None => false,
    };

    registries::with_port(
        port,
        |motor: &mut Motor| {
            // TODO: add device error exception
            motor.set_gearset(gearset).unwrap();
            motor.set_direction(direction).unwrap();
        },
        |port| Motor::new(port, gearset, direction),
    );

    alloc_obj(MotorObj {
        base: ObjBase::new::<MotorObj>(),
        port,
    })
}
