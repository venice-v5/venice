pub mod state;

use micropython_rs::{
    const_dict,
    except::raise_type_error,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use self::state::ControllerStateObj;
use super::raise_device_error;
use crate::{
    devices,
    fun::{fun1_from_fn, fun2_from_fn},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::ControllerGuard,
};

#[repr(C)]
pub struct ControllerObj {
    base: ObjBase<'static>,
    guard: ControllerGuard<'static>,
}

static CONTROLLER_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Controller))
        .set_make_new(make_new_from_fn!(controller_make_new))
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(UPDATE_INTERVAL_MS) => Obj::from_int(25),
            qstr!(MAX_COLUMNS) => Obj::from_int(19),
            qstr!(MAX_LINES) => Obj::from_int(3),

            qstr!(read_state) => Obj::from_static(&fun1_from_fn!(controller_read_state, &ControllerObj)),
            qstr!(rumble) => Obj::from_static(&fun2_from_fn!(controller_rumble, &ControllerObj, &[u8])),
            qstr!(free) => Obj::from_static(&fun1_from_fn!(controller_free, &ControllerObj))
        ]);

unsafe impl ObjTrait for ControllerObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_OBJ_TYPE.as_obj_type();
}

fn controller_make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, _args: &[Obj]) -> Obj {
    if n_pos != 0 || n_kw != 0 {
        raise_type_error(token().unwrap(), "function does not accept arguments");
    }

    let guard = devices::lock_controller();
    alloc_obj(ControllerObj {
        base: ObjBase::new(ControllerObj::OBJ_TYPE),
        guard,
    })
}

fn controller_read_state(this: &ControllerObj) -> Obj {
    let state = this
        .guard
        .borrow()
        .state()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    alloc_obj(ControllerStateObj::new(state))
}

fn controller_rumble(this: &ControllerObj, pattern: &[u8]) -> Obj {
    // TODO: execute in event loop
    // sound to unwrap because python strings are always valid UTF-8
    let pattern_str = str::from_utf8(pattern).unwrap();
    let _result = this.guard.borrow_mut().rumble(pattern_str);

    Obj::NONE
}

fn controller_free(this: &ControllerObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
