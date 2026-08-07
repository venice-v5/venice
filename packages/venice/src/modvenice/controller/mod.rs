pub mod id;
pub mod state;

use std::cell::RefCell;

use argparse::{ArgParser, Args, DefaultParser, IntParser, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{Message, raise_stop_iteration, value_error},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vex_sdk_jumptable::{
    V5_ControllerId, V5_ControllerStatus, vexControllerConnectionStatusGet, vexControllerTextSet,
};
use vexide_devices::controller::{Controller, ControllerConnection, ControllerError, ControllerId};

use self::state::ControllerStateObj;
use crate::{
    alloc::Gc,
    devices,
    modvenice::{
        Exception, controller::id::ControllerIdObj, device_error, read_only_attr::read_only_attr,
    },
    registry::ControllerGuard,
};

/// V5 Controller.
///
/// This class allows you to read from the buttons and joysticks on a controller and write to the
/// controller's display. The read-only `id` attribute is the `ControllerId` selected at
/// construction. Only one live Venice binding may hold each controller at a time; call
/// `Controller.free` before reusing its ID.
#[class(qstr!(Controller))]
#[repr(C)]
pub struct ControllerObj {
    base: ObjBase,
    guard: ControllerGuard,
}

impl From<ControllerError> for Exception {
    fn from(value: ControllerError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

/// Represents the state of a controller's connection. Values are returned by
/// `Controller.get_connection`.
///
/// This associated class isn't currently exported from the `venice` module, so its named constants
/// aren't directly reachable even though returned values use this type.
#[class(qstr!(ControllerConnection))]
#[repr(C)]
pub struct ControllerConnectionObj {
    base: ObjBase,
    connection: ControllerConnection,
}

impl ControllerConnectionObj {
    const fn new(connection: ControllerConnection) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            connection,
        }
    }
}

#[class_methods]
impl ControllerConnectionObj {
    /// No controller is connected.
    #[constant]
    pub const OFFLINE: &Self = &Self::new(ControllerConnection::Offline);
    /// Controller is tethered through a wired Smart Port connection.
    #[constant]
    pub const TETHERED: &Self = &Self::new(ControllerConnection::Tethered);
    /// Controller is wirelessly connected over a VEXNet radio.
    #[constant]
    pub const VEX_NET: &Self = &Self::new(ControllerConnection::VexNet);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.connection {
            ControllerConnection::Offline => "ControllerConnection.OFFLINE",
            ControllerConnection::Tethered => "ControllerConnection.TETHERED",
            ControllerConnection::VexNet => "ControllerConnection.VEX_NET",
        });
    }
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
/// An awaitable that completes once a write to the controller screen or vibration motor has been
/// performed.
///
/// This awaitable waits until the controller is able to accept a new write and fails if the controller
/// is disconnected or if the requested write is bad. Users receive it from `Controller.rumble`,
/// `Controller.clear_line`, `Controller.clear_screen`, or `Controller.set_text` rather than constructing
/// it directly. Awaiting it returns `None`.
///
/// # Raises
///
/// - `DeviceError`: If the controller is not connected.
/// - `ValueError`: If a requested line or column is outside its visible range.
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
        let this = self_in.as_obj::<ControllerFutureObj>();
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

        Obj::NONE
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
    /// The update rate of the controller, in milliseconds. Its value is 25.
    #[constant]
    const UPDATE_INTERVAL_MS: i32 = Controller::UPDATE_INTERVAL.as_millis() as i32;
    /// Maximum number of characters that can be drawn to a text line. Its value is 19.
    #[constant]
    const MAX_COLUMNS: i32 = Controller::MAX_COLUMNS as i32;
    /// Number of available text lines on the controller before clearing the screen. Its value is 3.
    #[constant]
    const MAX_LINES: i32 = Controller::MAX_LINES as i32;

    /// Creates a new controller selected by `id`.
    ///
    /// `id` defaults to `ControllerId.PRIMARY`.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If that controller ID is already in use by another `Controller` binding.
    #[make_new]
    #[stub(sig = "(self, id: ControllerId = ControllerId.PRIMARY) -> None")]
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
    #[stub(attrs = ["id: ControllerId"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "id" => Obj::from_static(match self.guard.borrow().id() {
                ControllerId::Primary => ControllerIdObj::PRIMARY,
                ControllerId::Partner => ControllerIdObj::PARTNER,
            }),
            _ => return,
        })
    }

    /// Returns the current state of all buttons and joysticks on the controller.
    ///
    /// # Note
    ///
    /// If the current competition mode is not driver control, this method raises an exception.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///
    ///     while True:
    ///         state = controller.read_state()
    ///
    ///         print("Left Stick X:", state.left_stick.x)
    ///         if state.button_a.is_now_pressed:
    ///             print("Button A was just pressed!")
    ///         if state.button_x.is_pressed:
    ///             print("Button X is pressed!")
    ///         if state.button_b.is_released:
    ///             print("Button B is released!")
    ///
    ///         await vasyncio.Sleep(Controller.UPDATE_INTERVAL_MS, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If access to controller data is restricted by competition control, or the
    ///   controller is not connected.
    #[method]
    fn read_state(&self) -> Result<ControllerStateObj, Exception> {
        let state = self.guard.borrow().state()?;
        Ok(ControllerStateObj::new(state))
    }

    /// Returns the controller's connection type.
    ///
    /// The result is a `ControllerConnection`. That associated class is currently missing from the
    /// root runtime dictionary, so its named constants aren't directly importable.
    #[method]
    #[stub(sig = "(self) -> ControllerConnection")]
    fn get_connection(&self) -> Obj {
        match self.guard.borrow().connection() {
            ControllerConnection::Offline => Obj::from_static(ControllerConnectionObj::OFFLINE),
            ControllerConnection::Tethered => Obj::from_static(ControllerConnectionObj::TETHERED),
            ControllerConnection::VexNet => Obj::from_static(ControllerConnectionObj::VEX_NET),
        }
    }

    /// Returns the controller's battery capacity as a float in the interval [0.0, 1.0].
    ///
    /// # Examples
    ///
    /// Print the controller's battery capacity:
    ///
    /// ```python
    /// from venice import *
    ///
    /// controller = Controller()
    /// print("Controller battery capacity:", controller.get_battery_capacity())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected.
    #[method]
    fn get_battery_capacity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().battery_capacity()? as f32)
    }

    /// Returns the controller's battery level.
    ///
    /// # Examples
    ///
    /// Print a warning if the controller battery is low:
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///     while True:
    ///         # If the controller isn't connected, it may as well be dead.
    ///         try:
    ///             battery_level = controller.get_battery_level()
    ///         except DeviceError:
    ///             battery_level = 0
    ///         if battery_level < 10:
    ///             print("WARNING: Controller battery is low!")
    ///         await vasyncio.Sleep(Controller.UPDATE_INTERVAL_MS, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected.
    #[method]
    fn get_battery_level(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().battery_level()?)
    }

    /// Returns the controller's flags.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected.
    #[method]
    fn get_flags(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().flags()?)
    }

    /// Sends a rumble `pattern` to the controller's vibration motor.
    ///
    /// This method takes a string consisting of the characters `'.'`, `'-'`, and `' '`, where dots are
    /// short rumbles, dashes are long rumbles, and spaces are pauses. Maximum supported length is 8
    /// characters. The operation isn't sent until the returned `ControllerFuture` is awaited.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///     await controller.rumble(". -. -.")
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `pattern` contains a NUL character.
    /// - `DeviceError`: When awaited, if the controller is not connected.
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

    /// Sends a rumble `pattern` to the controller's vibration motor.
    ///
    /// Unlike `Controller.rumble`, this method will fail if the controller screen is busy.
    ///
    /// This method takes a string consisting of the characters `'.'`, `'-'`, and `' '`, where dots are
    /// short rumbles, dashes are long rumbles, and spaces are pauses. Maximum supported length is 8
    /// characters. An embedded NUL isn't safely reported as a Python exception by this immediate
    /// method; use `Controller.rumble` when input may contain one.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// controller = Controller()
    /// controller.try_rumble(". -. -.")
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected or the screen is busy.
    #[method]
    fn try_rumble(&self, pattern: &str) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_rumble(pattern)?)
    }

    /// Clears the contents of a specific text `line`, waiting until the controller successfully clears
    /// the line.
    ///
    /// Lines are 1-indexed.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so calls to this function at intervals faster than 10ms
    /// on a wired connection or 50ms over VEXNet will take longer to complete.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///
    ///     # Write to line 1.
    ///     await controller.set_text("Hello, world!", 1, 1)
    ///
    ///     await vasyncio.Sleep(500, MILLIS)
    ///
    ///     # Clear line 1.
    ///     await controller.clear_line(1)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `line` is outside 1 through `Controller.MAX_LINES`.
    /// - `DeviceError`: If the controller is not connected.
    #[method]
    #[stub(sig = "(self, line: int) -> ControllerFuture")]
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

    /// Attempts to clear the contents of a specific text `line`.
    ///
    /// Lines are 1-indexed. Unlike `Controller.clear_line`, this method will fail if the controller screen
    /// is busy.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms on a wired connection or
    /// 50ms over VEXNet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///
    ///     # Write to line 1.
    ///     await controller.set_text("Hello, world!", 1, 1)
    ///
    ///     await vasyncio.Sleep(500, MILLIS)
    ///
    ///     # Clear line 1.
    ///     controller.try_clear_line(1)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `line` is outside 1 through `Controller.MAX_LINES`.
    /// - `DeviceError`: If the controller is not connected or the screen is busy.
    #[method]
    #[stub(sig = "(self, line: int) -> None")]
    fn try_clear_line(&self, line: Line) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_clear_line(line.0 as u8)?)
    }

    /// Clears the whole screen, waiting until the controller successfully clears the screen.
    ///
    /// This includes the default widget displayed by the controller if it hasn't already been cleared.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so calls to this function at intervals faster than 10ms
    /// on a wired connection or 50ms over VEXNet will take longer to complete.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///
    ///     # Remove the default widget on the controller screen that displays match time.
    ///     await controller.clear_screen()
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected.
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

    /// Clears the whole screen, including the default widget displayed by the controller if it hasn't
    /// already been cleared.
    ///
    /// Unlike `Controller.clear_screen`, this method will fail if the controller screen is busy.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms on a wired connection or
    /// 50ms over VEXNet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// controller = Controller()
    ///
    /// # Remove the default widget on the controller screen that displays match time.
    /// controller.try_clear_screen()
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the controller is not connected or the screen is busy.
    #[method]
    fn try_clear_screen(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().try_clear_screen()?)
    }

    /// Sets the text contents at a specific `line`/`column` offset, waiting until the controller
    /// successfully writes the `text`.
    ///
    /// Both lines and columns are 1-indexed.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so calls to this function at intervals faster than 10ms
    /// on a wired connection or 50ms over VEXNet will take longer to complete.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     controller = Controller()
    ///     await controller.set_text("Hello, world!", 1, 1)
    ///     await controller.set_text("Hello, world!", 2, 1)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `line` or `column` is outside its visible range, or `text` contains a NUL
    ///   character.
    /// - `DeviceError`: If the controller is not connected.
    #[method(ty = var(min = 4))]
    #[stub(sig = "(self, text: str, line: int, column: int) -> ControllerFuture")]
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

    /// Sets the `text` contents at a specific `line`/`column` offset.
    ///
    /// Both lines and columns are 1-indexed. An embedded NUL in `text` isn't safely reported as a
    /// Python exception by this immediate method; use `Controller.set_text` when input may contain
    /// one.
    ///
    /// Unlike `Controller.set_text`, this method will fail if the controller screen is busy.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms on a wired connection or
    /// 50ms over VEXNet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// controller = Controller()
    /// controller.try_set_text("Hello, world!", 1, 1)
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `line` or `column` is outside its visible range.
    /// - `DeviceError`: If the controller is not connected or the screen is busy.
    #[method(ty = var(min = 4))]
    #[stub(sig = "(self, text: str, line: int, column: int) -> None")]
    fn try_set_text(args: &[Obj]) -> Result<(), Exception> {
        let (this, text, line, column) = set_text_prelude(args)?;

        Ok(this
            .guard
            .borrow_mut()
            .try_set_text(text, line.0 as u8, column.0 as u8)?)
    }

    /// Releases this binding so another `Controller` can use the same controller ID.
    ///
    /// The object is unusable afterward.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the controller has already been freed.
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
