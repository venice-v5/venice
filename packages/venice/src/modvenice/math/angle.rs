use micropython_rs::{const_dict, except::raise_type_error, fun::{Fun0, Fun1}, init::token, obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags}};
use vexide_devices::math::Angle;

use crate::{args::{ArgType, Args}, obj::alloc_obj, qstrgen::qstr};


#[repr(C)]
pub struct AngleObj {
    base: ObjBase,
    angle: Angle
}

static ANGLE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Gearset))
    .set_slot_make_new(angle_make_new);

unsafe impl ObjTrait for AngleObj {
    const OBJ_TYPE: &ObjType = ANGLE_OBJ_TYPE.as_obj_type();
}


extern "C" fn angle_make_new(_: *const ObjType, n_pos: usize, n_kw: usize, ptr: *const Obj) -> Obj {
    let token = token().unwrap();
    let mut args = unsafe { Args::from_ptr(n_pos, n_kw, ptr) }.reader(token);
    args.assert_npos(0, 0).assert_nkw(1, 1);

    let arg = args.next_kw(ArgType::Float);
    let val = arg.value.as_float() as f64;
    let angle = match arg.kw {
        b"rad" => Angle::from_radians(val),
        b"deg" => Angle::from_degrees(val),
        _ => raise_type_error(
            token,
            "invalid keyword argument (expected either `deg=n` or `rad=n`)",
        ),
    };

    alloc_obj(AngleObj {
        base: ObjBase::new(ANGLE_OBJ_TYPE.as_obj_type()),
        angle
    })
}
