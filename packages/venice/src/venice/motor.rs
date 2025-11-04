use micropython_rs::{
    except::raise_value_error,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::{
    math::Direction,
    smart::motor::{Gearset, Motor},
};

use crate::{
    args::{ArgType, ArgValue, Args},
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

unsafe impl ObjTrait for MotorObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = MOTOR_OBJ_TYPE.as_obj_type();
}

fn gearset_from_str(str: &[u8]) -> Option<Gearset> {
    match str {
        b"red" => Some(Gearset::Red),
        b"green" => Some(Gearset::Green),
        b"blue" => Some(Gearset::Blue),
        _ => None,
    }
}

fn direction_from_str(str: &[u8]) -> Option<Direction> {
    match str {
        b"forward" => Some(Direction::Forward),
        b"reverse" => Some(Direction::Reverse),
        _ => None,
    }
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

    let gearset =
        gearset_from_str(args.next_positional(ArgType::Str).as_str()).unwrap_or_else(|| {
            raise_value_error(
                token,
                "invalid gearset (expected one of 'red', 'green', or 'blue')",
            )
        });

    let direction = direction_from_str(
        args.next_positional_or(ArgType::Str, ArgValue::Str(b"forward"))
            .as_str(),
    )
    .unwrap_or_else(|| {
        raise_value_error(
            token,
            "invalid direction (expected one of 'forward' or 'reverse')",
        )
    });

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
