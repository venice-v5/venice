pub mod state;

use micropython_rs::{
    const_dict,
    except::raise_type_error,
    fun::{Fun1, Fun2},
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::V5MotorControlMode;

use self::state::ControllerStateObj;
use super::raise_device_error;
use crate::{
    args::ArgType,
    devices,
    fun::{fun1_from_fn, fun2_from_fn},
    obj::alloc_obj,
    qstrgen::qstr,
};

#[repr(C)]
pub struct ControllerObj {
    base: ObjBase<'static>,
}

static CONTROLLER_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Controller))
        .set_make_new(make_new_from_fn!(controller_make_new))
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(read_state) => Obj::from_static(&fun1_from_fn!(fn controller_read_state(&ControllerObj))),
            qstr!(rumble) => Obj::from_static(&fun2_from_fn!(fn controller_rumble(&ControllerObj, &[u8]))),
        ]);

unsafe impl ObjTrait for ControllerObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_OBJ_TYPE.as_obj_type();
}

static CONTROLLER_OBJ: Obj = Obj::from_static(&ControllerObj {
    base: ObjBase::new(ControllerObj::OBJ_TYPE),
});

fn controller_make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, _args: &[Obj]) -> Obj {
    if n_pos != 0 || n_kw != 0 {
        raise_type_error(token().unwrap(), "function does not accept arguments");
    }

    CONTROLLER_OBJ
}

fn controller_read_state(_: &ControllerObj) -> Obj {
    let state = devices::try_lock_controller()
        .unwrap()
        .state()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    alloc_obj(ControllerStateObj::new(state))
}

fn controller_rumble(_: &ControllerObj, pattern: &[u8]) -> Obj {
    // TODO: execute in event loop
    let _result = devices::try_lock_controller()
        .unwrap()
        .rumble(str::from_utf8(pattern).unwrap());

    Obj::NONE
}
