pub mod id;
pub mod state;

use micropython_rs::{
    const_dict,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::controller::Controller;

use self::state::ControllerStateObj;
use crate::{
    args::Args,
    devices,
    fun::{fun1, fun2},
    modvenice::{controller::id::ControllerIdObj, raise_port_error},
    obj::alloc_obj,
    qstrgen::qstr,
    registry::ControllerGuard,
};

#[repr(C)]
pub struct ControllerObj {
    base: ObjBase<'static>,
    guard: ControllerGuard<'static>,
}

static CONTROLLER_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Controller))
    .set_make_new(make_new_from_fn!(controller_make_new))
    .set_locals_dict(const_dict![
        qstr!(UPDATE_INTERVAL_MS) => Obj::from_int(Controller::UPDATE_INTERVAL.as_micros() as i32),
        qstr!(MAX_COLUMNS) => Obj::from_int(Controller::MAX_COLUMNS as i32),
        qstr!(MAX_LINES) => Obj::from_int(Controller::MAX_LINES as i32),

        qstr!(read_state) => Obj::from_static(&fun1!(controller_read_state, &ControllerObj)),
        qstr!(rumble) => Obj::from_static(&fun2!(controller_rumble, &ControllerObj, &[u8])),
        qstr!(free) => Obj::from_static(&fun1!(controller_free, &ControllerObj))
    ]);

unsafe impl ObjTrait for ControllerObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_OBJ_TYPE.as_obj_type();
}

fn controller_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(0, 1).assert_nkw(0, 0);

    let id_obj = reader.next_positional_or(&ControllerIdObj::PRIMARY);

    let guard = devices::lock_controller(id_obj.id());
    alloc_obj(ControllerObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn controller_read_state(this: &ControllerObj) -> Obj {
    let state = this
        .guard
        .borrow()
        .state()
        .unwrap_or_else(|e| raise_port_error!(e));
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
