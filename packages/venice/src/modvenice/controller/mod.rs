pub mod id;
pub mod state;

use std::cell::RefCell;

use argparse::{ArgParser, Args, DefaultParser, IntParser, error_msg};
use micropython_rs::{
    class, class_methods,
    except::{Message, raise_stop_iteration, value_error},
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
        Exception, controller::id::ControllerIdObj, device_error, vasyncio::event_loop::WAKE_SIGNAL,
    },
    registry::ControllerGuard,
};

#[class(qstr!(Controller))]
#[repr(C)]
pub struct ControllerObj {
    base: ObjBase,
    guard: ControllerGuard<'static>,
}

impl From<ControllerError> for Exception {
    fn from(value: ControllerError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class(qstr!(ControllerConnection))]
#[repr(C)]
pub struct ControllerConnectionObj {
    base: ObjBase,
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
    #[constant]
    pub const OFFLINE: &Self = &Self::new();
    #[constant]
    pub const TETHERED: &Self = &Self::new();
    #[constant]
    pub const VEX_NET: &Self = &Self::new();
}

enum ControllerFuture {
    WaitingForIdle {
        line: u8,
        column: u8,
        text: Vec<u8, Gc>, // CString doesn't support custom allocators
        controller_id: ControllerId,
    },
    Complete,
}

// TODO: does this future need exclusive access to the controller as long as it lives?
#[class(qstr!(ControllerFuture))]
#[repr(C)]
pub struct ControllerFutureObj {
    base: ObjBase,
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

struct Line(u8);
#[derive(Default)]
struct LineParser;

impl<'a> ArgParser<'a> for LineParser {
    type Output = Line;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        IntParser::new(1..=Controller::MAX_LINES as i32)
            .parse(obj)
            .map(Line)
    }
}

impl DefaultParser<'_> for Line {
    type Parser = LineParser;
}

struct Column(u8);
#[derive(Default)]
struct ColumnParser;

impl<'a> ArgParser<'a> for ColumnParser {
    type Output = Column;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        IntParser::new(1..=Controller::MAX_COLUMNS as i32)
            .parse(obj)
            .map(Column)
    }
}

impl DefaultParser<'_> for Column {
    type Parser = ColumnParser;
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
        } = &*future
        {
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
                    Exception::from(e).raise(token());
                }
            }
        }

        Obj::from_static(&WAKE_SIGNAL)
    }
}

fn str_to_cstring_vec(str: &str, error_msg: impl Into<Message>) -> Vec<u8, Gc> {
    if str.find('\0').is_some() {
        value_error(error_msg.into()).raise(token());
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

fn set_text_prelude(args: &[Obj]) -> Result<(&ControllerObj, &str, Line, Column), Exception> {
    let mut reader = Args::new(args.len(), 0, args).reader();
    reader.assert_npos(4, 4);
    let this = reader.next_positional::<&ControllerObj>()?;
    let text = reader.next_positional::<&str>()?;
    let line = reader.next_positional::<Line>()?;
    let column = reader.next_positional::<Column>()?;

    Ok((this, text, line, column))
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
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 1).assert_nkw(0, 0);

        let id_obj = reader.next_positional_or(ControllerIdObj::PRIMARY)?;

        let guard = devices::lock_controller(id_obj.id());
        Ok(ControllerObj {
            base: ObjBase::new(ty),
            guard,
        })
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
    fn read_state(&self) -> Result<ControllerStateObj, Exception> {
        let state = self.guard.borrow().state()?;
        Ok(ControllerStateObj::new(state))
    }

    #[method]
    fn get_connection(&self) -> Obj {
        match self.guard.borrow().connection() {
            ControllerConnection::Offline => Obj::from_static(ControllerConnectionObj::OFFLINE),
            ControllerConnection::Tethered => Obj::from_static(ControllerConnectionObj::TETHERED),
            ControllerConnection::VexNet => Obj::from_static(ControllerConnectionObj::VEX_NET),
        }
    }

    #[method]
    fn get_battery_capacity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().battery_capacity()? as f32)
    }

    #[method]
    fn get_battery_level(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().battery_level()?)
    }

    #[method]
    fn get_flags(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().flags()?)
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
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_rumble(&self, pattern: &str) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_rumble(pattern)?)
    }

    #[method]
    fn clear_line(&self, line: Line) -> ControllerFutureObj {
        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line: line.0,
                column: 1,
                text: empty_cstring_vec(),
                controller_id: self.guard.borrow().id(),
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_clear_line(&self, line: Line) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_clear_line(line.0 as u8)?)
    }

    #[method]
    fn clear_screen(&self) -> ControllerFutureObj {
        ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line: 0,
                column: 1,
                text: empty_cstring_vec(),
                controller_id: self.guard.borrow().id(),
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        }
    }

    #[method]
    fn try_clear_screen(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_clear_screen()?)
    }

    #[method(ty = var(min = 4))]
    fn set_text(args: &[Obj]) -> Result<ControllerFutureObj, Exception> {
        let (this, text, line, column) = set_text_prelude(args)?;

        Ok(ControllerFutureObj {
            future: RefCell::new(ControllerFuture::WaitingForIdle {
                line: line.0,
                column: column.0,
                text: str_to_cstring_vec(text, c"text has forbidden nul byte"),
                controller_id: this.guard.borrow().id(),
            }),
            base: ObjBase::new(ControllerFutureObj::OBJ_TYPE),
        })
    }

    #[method(ty = var(min = 4))]
    fn try_set_text(args: &[Obj]) -> Result<(), Exception> {
        let (this, text, line, column) = set_text_prelude(args)?;

        Ok(this
            .guard
            .borrow_mut()
            .try_set_text(text, line.0 as u8, column.0 as u8)?)
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
