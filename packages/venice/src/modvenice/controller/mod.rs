pub mod id;
pub mod state;

use std::{cell::RefCell, ffi::CStr, ops::RangeInclusive};

use argparse::{Args, error_msg};
use micropython_rs::{
    class, class_methods,
    except::{raise_stop_iteration, raise_value_error},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vex_sdk::{
    V5_ControllerId, V5_ControllerStatus, vexControllerConnectionStatusGet, vexControllerTextSet,
};
use vexide_devices::controller::{Controller, ControllerConnection, ControllerError, ControllerId};

use self::state::ControllerStateObj;
use crate::{
    alloc::Gc,
    devices,
    modvenice::{
        controller::id::ControllerIdObj, raise_device_error, raise_port_error,
        vasyncio::event_loop::WAKE_SIGNAL,
    },
    registry::ControllerGuard,
};

#[class(qstr!(Controller))]
#[repr(C)]
pub struct ControllerObj {
    base: ObjBase<'static>,
    guard: ControllerGuard<'static>,
}

#[class(qstr!(MotorV5))]
#[repr(C)]
pub struct ControllerConnectionObj {
    base: ObjBase<'static>,
}

impl ControllerConnectionObj {
    const fn new() -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
        }
    }
}

#[class_methods]
impl ControllerConnectionObj {
    pub const OFFLINE: &Self = &Self::new();
    pub const TETHERED: &Self = &Self::new();
    pub const VEX_NET: &Self = &Self::new();
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
#[class(qstr!(ControllerFuture))]
#[repr(C)]
pub struct ControllerFutureObj {
    base: ObjBase<'static>,
    future: RefCell<ControllerFuture>,
}

fn validate_connection(id: ControllerId) -> Result<(), ControllerError> {
    if unsafe {
        vexControllerConnectionStatusGet(id.into()) == V5_ControllerStatus::kV5ControllerOffline
    } {
        return Err(ControllerError::Offline);
    }

    Ok(())
}

fn validate_line(line: &i32) {
    const LINE_RANGE: RangeInclusive<i32> = 1..=Controller::MAX_LINES as i32;
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

fn validate_column(column: &i32) {
    const COLUMN_RANGE: RangeInclusive<i32> = 1..=Controller::MAX_COLUMNS as i32;
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
}

#[class_methods]
impl ControllerFutureObj {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
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
                validate_line(line);
            }

            validate_column(column);

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
                        raise_stop_iteration(token(), Obj::NONE);
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
}

fn str_to_cstring_vec(str: &str, error_msg: impl AsRef<CStr>) -> Vec<u8, Gc> {
    if str.find('\0').is_some() {
        raise_value_error(token(), error_msg);
    }

    let mut vec = Vec::with_capacity_in(str.len() + 1, Gc { token: token() });
    vec.extend_from_slice(str.as_bytes());
    vec.push(0);
    vec
}

fn empty_cstring_vec() -> Vec<u8, Gc> {
    let mut vec = Vec::new_in(Gc { token: token() });
    vec.push(0);
    vec
}

fn set_text_prelude(args: &[Obj]) -> (&ControllerObj, &str, i32, i32) {
    let mut reader = Args::new(args.len(), 0, args).reader(token());
    reader.assert_npos(4, 4);
    let this = reader.next_positional::<&ControllerObj>();
    let text = reader.next_positional::<&str>();
    let line = reader.next_positional::<i32>();
    let column = reader.next_positional::<i32>();

    (this, text, line, column)
}

#[class_methods]
impl ControllerObj {
    #[constant]
    const UPDATE_INTERVAL_MS: i32 = Controller::UPDATE_INTERVAL.as_millis() as i32;
    #[constant]
    const MAX_COLUMNS: i32 = Controller::MAX_COLUMNS as i32;
    #[constant]
    const MAX_LINES: i32 = Controller::MAX_LINES as i32;

    #[make_new]
    fn make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Self {
        let token = token();
        let mut reader = Args::new(n_pos, n_kw, args).reader(token);
        reader.assert_npos(0, 1).assert_nkw(0, 0);

        let id_obj = reader.next_positional_or(ControllerIdObj::PRIMARY);

        let guard = devices::lock_controller(id_obj.id());
        ControllerObj {
            base: ObjBase::new(ty),
            guard,
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "id" => Obj::from_static(match self.guard.borrow().id() {
                ControllerId::Primary => ControllerIdObj::PRIMARY,
                ControllerId::Partner => ControllerIdObj::PARTNER,
            }),
            _ => return,
        })
    }

    #[method]
    fn read_state(&self) -> ControllerStateObj {
        let state = self
            .guard
            .borrow()
            .state()
            .unwrap_or_else(|e| raise_port_error!(e));
        ControllerStateObj::new(state)
    }

    #[method]
    fn connection(&self) -> Obj {
        match self.guard.borrow().connection() {
            ControllerConnection::Offline => Obj::from_static(ControllerConnectionObj::OFFLINE),
            ControllerConnection::Tethered => Obj::from_static(ControllerConnectionObj::TETHERED),
            ControllerConnection::VexNet => Obj::from_static(ControllerConnectionObj::VEX_NET),
        }
    }

    #[method]
    fn battery_capacity(&self) -> f32 {
        self.guard
            .borrow()
            .battery_capacity()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn battery_level(&self) -> i32 {
        self.guard
            .borrow()
            .battery_level()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn flags(&self) -> i32 {
        self.guard
            .borrow()
            .flags()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn rumble(&self, pattern: &str) -> ControllerFutureObj {
        let text = str_to_cstring_vec(pattern, c"rumble pattern has forbidden nul byte");

        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line: 4,
                column: 1,
                text,
                controller_id: self.guard.borrow().id(),
                enforce_visible: false,
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_rumble(&self, pattern: &str) {
        self.guard
            .borrow_mut()
            .try_rumble(pattern)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn clear_line(&self, line: i32) -> ControllerFutureObj {
        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line,
                column: 1,
                text: empty_cstring_vec(),
                controller_id: self.guard.borrow().id(),
                enforce_visible: true,
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_clear_line(&self, line: i32) {
        validate_line(&line);
        self.guard
            .borrow_mut()
            .try_clear_line(line as u8)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn clear_screen(&self) -> ControllerFutureObj {
        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line: 0,
                column: 1,
                text: empty_cstring_vec(),
                controller_id: self.guard.borrow().id(),
                enforce_visible: false,
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_clear_screen(&self) {
        self.guard
            .borrow_mut()
            .try_clear_screen()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method(ty = var(min = 4))]
    fn set_text(args: &[Obj]) -> ControllerFutureObj {
        let (this, text, line, column) = set_text_prelude(args);

        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line,
                column,
                text: str_to_cstring_vec(text, c"text has forbidden nul byte"),
                controller_id: this.guard.borrow().id(),
                enforce_visible: false,
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method(ty = var(min = 4))]
    fn try_set_text(args: &[Obj]) {
        let (this, text, line, column) = set_text_prelude(args);

        validate_line(&line);
        validate_column(&column);

        this.guard
            .borrow_mut()
            .try_set_text(text, line as u8, column as u8)
            .unwrap_or_else(|e| raise_port_error!(e)); // technically not PortError but the macro works
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
