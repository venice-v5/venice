use micropython_rs::{
    const_dict,
    except::{raise_type_error, raise_value_error},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use super::direction::Direction;
use crate::{obj::alloc_obj, qstrgen::qstr};

#[repr(C)]
pub struct Motor {
    base: ObjBase,
    port: u8,
    gearset: Gearset,
    direction: Direction,
    exp: bool,
}

pub static MOTOR_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Motor)).set_slot_make_new(motor_make_new);

enum Gearset {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl Gearset {
    pub fn from_int(int: i32) -> Option<Self> {
        match int {
            0 => Some(Self::Red),
            1 => Some(Self::Green),
            2 => Some(Self::Blue),
            _ => None,
        }
    }
}

pub static GEARSET_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(RED) => Obj::from_small_int(Gearset::Red as i32),
        qstr!(GREEN) => Obj::from_small_int(Gearset::Green as i32),
        qstr!(BLUE) => Obj::from_small_int(Gearset::Blue as i32),
    ]);

unsafe impl ObjTrait for Motor {
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
    let port = args[0]
        .as_small_int()
        .unwrap_or_else(|| raise_type_error(token, "expected integer for port number"));
    if port < 1 || port > 21 {
        raise_value_error(token, "port number must be between 1 and 21");
    }

    let gearset_int = args[1]
        .as_small_int()
        .unwrap_or_else(|| raise_type_error(token, "expected Gearset object"));
    let gearset = Gearset::from_int(gearset_int)
        .unwrap_or_else(|| raise_type_error(token, "expected Gearset object"));

    let direction = match args.get(2) {
        Some(d) => Direction::from_int(
            d.as_small_int()
                .unwrap_or_else(|| raise_type_error(token, "expected Direction object")),
        )
        .unwrap_or_else(|| raise_type_error(token, "expected Direction object")),
        None => Direction::Reverse,
    };

    let exp = match args.get(3) {
        Some(e) => e
            .as_bool()
            .unwrap_or_else(|| raise_type_error(token, "expected bool")),
        None => false,
    };

    alloc_obj(Motor {
        base: ObjBase::new::<Motor>(),
        port: port as u8,
        gearset,
        direction,
        exp,
    })
}
