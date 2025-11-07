pub mod state;

use micropython_rs::{
    const_dict,
    except::raise_type_error,
    fun::{Fun1, Fun2},
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};

use self::state::ControllerStateObj;
use crate::{args::ArgType, devices, modvenice::raise_device_error, obj::alloc_obj, qstrgen::qstr};

#[repr(C)]
pub struct ControllerObj {
    base: ObjBase,
}

static CONTROLLER_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Controller))
    .set_slot_make_new(controller_make_new)
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(read_state) => Obj::from_static(&Fun1::new(controller_read_state)),
        qstr!(rumble) => Obj::from_static(&Fun2::new(controller_rumble)),
    ]);

unsafe impl ObjTrait for ControllerObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_OBJ_TYPE.as_obj_type();
}

static CONTROLLER_OBJ: Obj = Obj::from_static(&ControllerObj {
    base: ObjBase::new(ControllerObj::OBJ_TYPE),
});

extern "C" fn controller_make_new(
    _: *const ObjType,
    n_args: usize,
    n_kw: usize,
    _: *const Obj,
) -> Obj {
    if n_args != 0 || n_kw != 0 {
        raise_type_error(token().unwrap(), "function does not accept arguments");
    }

    CONTROLLER_OBJ
}

extern "C" fn controller_read_state(_self_in: Obj) -> Obj {
    let state = devices::try_lock_controller()
        .unwrap()
        .state()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    alloc_obj(ControllerStateObj::new(state))
}

extern "C" fn controller_rumble(_self_in: Obj, pattern_obj: Obj) -> Obj {
    let token = token().unwrap();
    let pattern = pattern_obj.get_str().unwrap_or_else(|| {
        raise_type_error(
            token,
            format!(
                "expected <str> for argument 1, found <{}>",
                ArgType::of(&pattern_obj)
            ),
        )
    });

    // TODO: execute in event loop
    let _result = devices::try_lock_controller()
        .unwrap()
        .rumble(str::from_utf8(pattern).unwrap());

    Obj::NONE
}
