pub mod id;
pub mod state;

use std::{cell::RefCell, ffi::CStr, ops::RangeInclusive};

use micropython_rs::{
    attr_from_fn, const_dict,
    except::raise_value_error,
    init::token,
    make_new_from_fn,
    obj::{AttrOp, Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
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
    fun::{fun_var, fun1, fun2},
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
    .set_attr(attr_from_fn!(controller_attr))
    .set_locals_dict(const_dict![
        qstr!(UPDATE_INTERVAL_MS) => Obj::from_int(Controller::UPDATE_INTERVAL.as_micros() as i32),
        qstr!(MAX_COLUMNS) => Obj::from_int(Controller::MAX_COLUMNS as i32),
        qstr!(MAX_LINES) => Obj::from_int(Controller::MAX_LINES as i32),

        qstr!(read_state) => Obj::from_static(&fun1!(controller_read_state, &ControllerObj)),

        qstr!(rumble) => Obj::from_static(&fun2!(controller_rumble, &ControllerObj, &str)),
        qstr!(clear_line) => Obj::from_static(&fun2!(controller_clear_line, &ControllerObj, i32)),
        qstr!(clear_screen) => Obj::from_static(&fun1!(controller_clear_screen, &ControllerObj)),
        qstr!(set_text) => Obj::from_static(&fun_var!(controller_set_text)),

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

fn controller_attr(this: &ControllerObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else { return };
    result.return_value(match attr.as_str() {
        "id" => Obj::from_static(match this.guard.borrow().id() {
            ControllerId::Primary => &ControllerIdObj::PRIMARY,
            ControllerId::Partner => &ControllerIdObj::PARTNER,
        }),
        _ => return,
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
        line: i32,
        column: i32,
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
        const LINE_RANGE: RangeInclusive<i32> = 1..=Controller::MAX_LINES as i32;
        const COLUMN_RANGE: RangeInclusive<i32> = 1..=Controller::MAX_COLUMNS as i32;

        if *enforce_visible {
            if !LINE_RANGE.contains(line) {
                raise_value_error(
                    token(),
                    error_msg!(
                        "line number ({line}) must be between ({}) and ({})",
                        LINE_RANGE.start(),
                        LINE_RANGE.end(),
                    ),
                );
            }
        }

        if !COLUMN_RANGE.contains(column) {
            raise_value_error(
                token(),
                error_msg!(
                    "column number ({column}) must be between ({}) and ({})",
                    COLUMN_RANGE.start(),
                    COLUMN_RANGE.end(),
                ),
            )
        }

        match validate_connection(*controller_id) {
            Ok(()) => {
                let id = V5_ControllerId::from(*controller_id);

                let result = unsafe {
                    vexControllerTextSet(
                        u32::from(id.0),
                        *line as u32,
                        (*column - 1) as u32,
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

fn str_to_cstring_vec(str: &str, error_msg: impl AsRef<CStr>) -> Vec<u8, Gc> {
    if let Some(_) = str.find('\0') {
        raise_value_error(token(), error_msg);
    }

    let mut vec = Vec::with_capacity_in(str.len() + 1, Gc { token: token() });
    vec.extend_from_slice(str.as_bytes());
    vec.push(0);
    vec
}

fn controller_rumble(this: &ControllerObj, pattern: &str) -> Obj {
    let text = str_to_cstring_vec(pattern, c"rumble pattern has forbidden nul byte");

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

fn empty_cstring_vec() -> Vec<u8, Gc> {
    let mut vec = Vec::new_in(Gc { token: token() });
    vec.push(0);
    vec
}

fn controller_clear_line(this: &ControllerObj, line: i32) -> Obj {
    alloc_obj(ControllerFutureObj {
        future: RefCell::new(ControllerFuture::WaitingForIdle {
            line,
            column: 1,
            text: empty_cstring_vec(),
            controller_id: this.guard.borrow().id(),
            enforce_visible: true,
        }),
        base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
    })
}

fn controller_clear_screen(this: &ControllerObj) -> Obj {
    alloc_obj(ControllerFutureObj {
        future: RefCell::new(ControllerFuture::WaitingForIdle {
            line: 0,
            column: 1,
            text: empty_cstring_vec(),
            controller_id: this.guard.borrow().id(),
            enforce_visible: false,
        }),
        base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
    })
}

fn controller_set_text(args: &[Obj]) -> Obj {
    let mut reader = Args::new(args.len(), 0, args).reader(token());
    reader.assert_npos(4, 4);
    let this = reader.next_positional::<&ControllerObj>();
    let text = reader.next_positional::<&str>();
    let line = reader.next_positional::<i32>();
    let column = reader.next_positional::<i32>();

    alloc_obj(ControllerFutureObj {
        future: RefCell::new(ControllerFuture::WaitingForIdle {
            line,
            column,
            text: str_to_cstring_vec(text, c"text has forbidden nul byte"),
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
