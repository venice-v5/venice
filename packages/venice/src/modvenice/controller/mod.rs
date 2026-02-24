pub mod id;
pub mod state;

use std::cell::RefCell;

use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::{
    V5_ControllerId, V5_ControllerStatus, vexControllerConnectionStatusGet, vexControllerTextSet,
};
use vexide_devices::controller::{Controller, ControllerError, ControllerId};

use self::state::ControllerStateObj;
use crate::{
    alloc::Gc,
    args::Args,
    devices,
    error_msg::error_msg,
    fun::{fun1, fun2},
    modvenice::{
        controller::id::ControllerIdObj, raise_device_error, raise_port_error,
        vasyncio::event_loop::WAKE_SIGNAL,
    },
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
        qstr!(rumble) => Obj::from_static(&fun2!(controller_rumble, &ControllerObj, &str)),
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

enum ControllerFuture {
    WaitingForIdle {
        line: u8,
        column: u8,
        text: Vec<u8, Gc>, // CString doesn't support custom allocators
        controller_id: ControllerId,
        enforce_visible: bool,
    },
    Complete,
}

// TODO: does this future need exclusive access to the controller as long as it lives?
#[repr(C)]
pub struct ControllerFutureObj {
    base: ObjBase<'static>,
    future: RefCell<ControllerFuture>,
}

pub static CONTROLLER_FUTURE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(ControllerFuture))
        .set_iter(Iter::IterNext(controller_future_iternext));

unsafe impl ObjTrait for ControllerFutureObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_FUTURE_OBJ_TYPE.as_obj_type();
}

fn validate_connection(id: ControllerId) -> Result<(), ControllerError> {
    if unsafe {
        vexControllerConnectionStatusGet(id.into()) == V5_ControllerStatus::kV5ControllerOffline
    } {
        return Err(ControllerError::Offline);
    }

    Ok(())
}

extern "C" fn controller_future_iternext(self_in: Obj) -> Obj {
    let this = self_in.try_as_obj::<ControllerFutureObj>().unwrap();
    let mut future = this.future.borrow_mut();

    if let ControllerFuture::WaitingForIdle {
        line,
        column,
        text,
        controller_id,
        enforce_visible,
    } = &*future
    {
        if *enforce_visible {
            if *line == 0 || *line > Controller::MAX_LINES as u8 {
                raise_value_error(
                    token(),
                    error_msg!(
                        "line number ({line}) is greater than the maximum number of lines ({})",
                        Controller::MAX_COLUMNS
                    ),
                );
            }
        }

        if *column != 0 && *column <= Controller::MAX_COLUMNS as u8 {
            raise_value_error(
                token(),
                error_msg!(
                    "Invalid column number ({column}) is greater than the maximum number of columns ({})",
                    Controller::MAX_COLUMNS
                ),
            )
        }

        match validate_connection(*controller_id) {
            Ok(()) => {
                let id = V5_ControllerId::from(*controller_id);

                let result = unsafe {
                    vexControllerTextSet(
                        u32::from(id.0),
                        u32::from(*line),
                        u32::from(*column - 1),
                        text.as_ptr().cast(),
                    )
                };

                if result == 1 {
                    *future = ControllerFuture::Complete;
                }
            }
            Err(e) => {
                *future = ControllerFuture::Complete;
                raise_device_error(token(), error_msg!("{e}"));
            }
        }
    }

    Obj::from_static(&WAKE_SIGNAL)
}

fn str_to_cstring_vec(str: &str) -> Result<Vec<u8, Gc>, ()> {
    if let Some(_) = str.find('\0') {
        return Err(());
    }

    let mut vec = Vec::with_capacity_in(str.len() + 1, Gc { token: token() });
    vec.extend_from_slice(str.as_bytes());
    vec.push(0);
    Ok(vec)
}

fn controller_rumble(this: &ControllerObj, pattern: &str) -> Obj {
    let text = str_to_cstring_vec(pattern)
        .unwrap_or_else(|_| raise_value_error(token(), c"rumble pattern has forbidden null byte"));

    alloc_obj(ControllerFutureObj {
        future: RefCell::new(ControllerFuture::WaitingForIdle {
            line: 4,
            column: 1,
            text,
            controller_id: this.guard.borrow().id(),
            enforce_visible: false,
        }),
        base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
    })
}

fn controller_free(this: &ControllerObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
