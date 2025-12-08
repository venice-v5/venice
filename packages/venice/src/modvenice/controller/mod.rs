pub mod state;

use std::{
    cell::RefCell,
    ffi::{CString, NulError},
    rc::Rc,
    sync::MutexGuard,
    task::Poll,
};

use micropython_rs::{
    const_dict,
    except::raise_type_error,
    fun::Fun2,
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk_jumptable as _;
use vex_sdk::{
    V5_ControllerId, V5_ControllerStatus, vexControllerConnectionStatusGet, vexControllerTextSet,
};
use vexide_devices::controller::{Controller, ControllerError, ControllerId};

use self::state::ControllerStateObj;
use super::raise_device_error;
use crate::{
    args::{ArgTrait, ArgValue}, devices, fun::{fun1_from_fn, fun2_from_fn}, modvenice::device_future::{DeviceFuture, DeviceFutureObj}, obj::alloc_obj, qstrgen::qstr
};

#[repr(C)]
pub struct ControllerObj {
    base: ObjBase<'static>,
    guard: MutexGuard<'static, Controller>,
}

static CONTROLLER_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Controller))
        .set_make_new(make_new_from_fn!(controller_make_new))
        .set_slot_locals_dict_from_static(&const_dict![
            qstr!(UPDATE_INTERVAL_MS) => Obj::from_int(25),
            qstr!(MAX_COLUMNS) => Obj::from_int(19),
            qstr!(MAX_LINES) => Obj::from_int(3),

            qstr!(read_state) => Obj::from_static(&fun1_from_fn!(controller_read_state, &ControllerObj)),
            qstr!(rumble) => Obj::from_static(&Fun2::new(controller_rumble)),
        ]);

unsafe impl ObjTrait for ControllerObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_OBJ_TYPE.as_obj_type();
}

fn controller_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, _args: &[Obj]) -> Obj {
    if n_pos != 0 || n_kw != 0 {
        raise_type_error(token().unwrap(), "function does not accept arguments");
    }

    let guard = devices::try_lock_controller().unwrap_or_else(|_| panic!("port is already in use"));

    alloc_obj(ControllerObj {
        base: ObjBase::new(ty),
        guard,
    })
}
fn controller_read_state(controller: &ControllerObj) -> Obj {
    let state = controller
        .guard
        .state()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    alloc_obj(ControllerStateObj::new(state))
}

extern "C" fn controller_rumble(controller_obj: Obj, pattern_obj: Obj) -> Obj {
    let controller_value = ArgValue::from_obj(&controller_obj);
    let pattern_value = ArgValue::from_obj(&pattern_obj);
    if controller_value.ty() != <&ControllerObj as ArgTrait>::ty() {
        raise_type_error(
            token().unwrap(),
            format!(
                "expected <{}> for argument #1, found <{}>",
                <&ControllerObj as ArgTrait>::ty(),
                controller_value.ty()
            ),
        );
    }
    if pattern_value.ty() != <&[u8] as ArgTrait>::ty() {
        raise_type_error(
            token().unwrap(),
            format!(
                "expected <{}> for argument #2, found <{}>",
                <&[u8] as ArgTrait>::ty(),
                pattern_value.ty()
            ),
        );
    }
    let pattern = unsafe { <&[u8] as ArgTrait>::from_arg_value(pattern_value).unwrap_unchecked() };

    DeviceFutureObj::new(DeviceFuture::ControllerScreenWrite(
        ControllerScreenWriteAwaitable::new(
            4,
            1,
            String::from_utf8(pattern.to_vec()).unwrap(),
            controller_obj,
            false,
        ),
    ))
}

enum ControllerScreenWriteAwaitableState {
    /// Waiting for the controller to be ready to accept a new write.
    WaitingForIdle {
        /// The line to write to.
        ///
        /// This is indexed like the SDK, with the first onscreen line being 1.
        line: u8,

        /// The column to write to.
        ///
        /// This **NOT** is indexed like the SDK. The first onscreen column is 1.
        column: u8,

        /// The text to write.
        text: Result<CString, NulError>,

        controller: Obj,

        /// Whether or not to enforce that this line is on screen.
        enforce_visible: bool,
    },

    /// The write has been completed.
    Complete {
        /// The result of the write.
        result: Result<(), ControllerError>,
    },
}

fn validate_connection(id: ControllerId) -> Result<(), ControllerError> {
    if unsafe {
        vexControllerConnectionStatusGet(id.into()) == V5_ControllerStatus::kV5ControllerOffline
    } {
        return Err(ControllerError::Offline);
    }

    Ok(())
}

pub struct ControllerScreenWriteAwaitable {
    state: RefCell<ControllerScreenWriteAwaitableState>,
}

impl ControllerScreenWriteAwaitable {
    pub fn new(line: u8, column: u8, text: String, controller: Obj, enforce_visible: bool) -> Self {
        Self {
            state: ControllerScreenWriteAwaitableState::WaitingForIdle {
                line,
                column,
                text: CString::new(text),
                controller,
                enforce_visible,
            }
            .into(),
        }
    }

    pub fn poll(&self) -> Option<Result<(), ControllerError>> {
        let mut state = self.state.borrow_mut();

        let transition = if let ControllerScreenWriteAwaitableState::WaitingForIdle {
            line,
            column,
            text,
            controller,
            enforce_visible,
        } = &*state
        {
            if *enforce_visible {
                assert!(
                    *line != 0 && *line <= Controller::MAX_LINES as u8,
                    "Invalid line number ({line}) is greater than the maximum number of lines ({})",
                    Controller::MAX_LINES
                );
            }

            assert!(
                *column != 0 && *column <= Controller::MAX_COLUMNS as u8,
                "Invalid column number ({column}) is greater than the maximum number of columns ({})",
                Controller::MAX_COLUMNS
            );

            let text = text
                .as_deref()
                .map_err(Clone::clone)
                .expect("A NUL (0x00) character was found in the text input string.");

            let controller: &ControllerObj = (*controller).try_to_obj().unwrap();
            let id = controller.guard.id();

            match validate_connection(id) {
                Ok(()) => {
                    let id = V5_ControllerId::from(id);

                    let result = unsafe {
                        vexControllerTextSet(
                            u32::from(id.0),
                            u32::from(*line),
                            u32::from(*column - 1),
                            text.as_ptr().cast(),
                        )
                    };

                    if result == 1 { Some(Ok(())) } else { None }
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        };

        if let Some(result) = transition {
            *state = ControllerScreenWriteAwaitableState::Complete { result };
        }

        if let ControllerScreenWriteAwaitableState::Complete { result } = *state {
            Some(result)
        } else {
            None
        }
    }
}
