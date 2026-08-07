use std::fmt::Write;

use argparse::{Args, PositionalError, error_msg};
use micropython_macros::{class, class_methods, fun};
use micropython_rs::{
    buffer::Buffer,
    const_dict,
    except::type_error,
    map::{Dict, Map},
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind, StringPrint},
    qstr::Qstr,
};
use vexide_devices::{
    color::Color,
    display::{
        Circle, Font, FontFamily, FontSize, Line, Rect, RenderMode, Text, TouchEvent, TouchState,
    },
    math::Point2,
};

use crate::{
    devices::lock_display,
    modvenice::{Exception, color::ColorObj, read_only_attr::read_only_attr},
};

pub const DISPLAY_DICT: &Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(display)),

    // classes
    qstr!(RenderMode) => Obj::from_static(RenderModeObj::OBJ_TYPE),
    qstr!(FontFamily) => Obj::from_static(FontFamilyObj::OBJ_TYPE),
    qstr!(FontSize) => Obj::from_static(FontSizeObj::OBJ_TYPE),
    qstr!(TouchEvent) => Obj::from_static(TouchEventObj::OBJ_TYPE),

    // drawing
    qstr!(draw_pixel) => draw_pixel_obj,
    qstr!(draw_line) => draw_line_obj,
    qstr!(draw_circle) => draw_circle_obj,
    qstr!(fill_circle) => fill_circle_obj,
    qstr!(draw_rect) => draw_rect_obj,
    qstr!(fill_rect) => fill_rect_obj,
    qstr!(draw_buffer) => draw_buffer_obj,
    qstr!(draw_text) => draw_text_obj,
    // scroll
    qstr!(scroll) => scroll_obj,
    qstr!(scroll_region) => scroll_region_obj,
    // render
    qstr!(set_render_mode) => set_render_mode_obj,
    qstr!(render) => render_obj,
    qstr!(erase) => erase_obj,
    // print
    qstr!(print) => print_obj,
    // touch
    qstr!(get_touch_status) => get_touch_status_obj,
    qstr!(is_now_pressed) => is_now_pressed_obj,
    qstr!(is_pressed) => is_pressed_obj,
    qstr!(is_released) => is_released_obj,
    qstr!(is_held) => is_held_obj,
];

/// The rendering mode for the VEX V5's display, available as `display.RenderMode`.
///
/// When using the display in `RenderMode.IMMEDIATE` mode, all draw operations will immediately show
/// up on the display. `RenderMode.DOUBLE_BUFFERED` mode instead applies draw operations onto an
/// intermediate buffer that can be swapped onto the display by calling `display.render`, thereby
/// preventing screen tearing. By default, the display uses `RenderMode.IMMEDIATE` mode.
///
/// # Note
///
/// `display.render` **MUST** be called for anything to appear on the display when using
/// `RenderMode.DOUBLE_BUFFERED` mode.
#[class(qstr!(RenderMode))]
#[repr(C)]
struct RenderModeObj {
    base: ObjBase,
    mode: RenderMode,
}

#[class_methods]
impl RenderModeObj {
    const fn new(mode: RenderMode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            mode,
        }
    }

    /// Draw operations are immediately applied to the display without the need to call `display.render`.
    #[constant]
    const IMMEDIATE: &Self = &Self::new(RenderMode::Immediate);
    /// Draw calls are affected on an intermediary display buffer, rather than directly drawn to the
    /// display. The intermediate buffer can later be applied to the display using `display.render`.
    ///
    /// This mode is necessary for preventing screen tearing when drawing at high speeds.
    #[constant]
    const DOUBLE_BUFFERED: &Self = &Self::new(RenderMode::DoubleBuffered);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.mode {
            RenderMode::Immediate => "RenderMode.IMMEDIATE",
            RenderMode::DoubleBuffered => "RenderMode.DOUBLE_BUFFERED",
        });
    }
}

/// The font family used by `display.draw_text`, available as `display.FontFamily`.
#[class(qstr!(FontFamily))]
struct FontFamilyObj {
    base: ObjBase,
    family: FontFamily,
}

#[class_methods]
impl FontFamilyObj {
    const fn new(family: FontFamily) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            family,
        }
    }

    /// A monospaced font which has a fixed width for each character.
    ///
    /// This font at full size is 49pt Noto Mono.
    #[constant]
    const MONOSPACE: &Self = &Self::new(FontFamily::Monospace);
    /// A proportional font which has a varying width for each character.
    ///
    /// This font at full size is 49pt Noto Sans.
    #[constant]
    const PROPORTIONAL: &Self = &Self::new(FontFamily::Proportional);
}

/// A fractional font scaling factor, available as `display.FontSize`.
///
/// The read-only `numerator` attribute is the numerator of the fractional font scale. The read-only
/// `denominator` attribute is the denominator of the fractional font scale. Use one of the predefined
/// constants or construct a custom positive fraction. The runtime doesn't currently reject a zero
/// `denominator`, but such a value isn't a valid font scale.
#[class(qstr!(FontSize))]
struct FontSizeObj {
    base: ObjBase,
    size: FontSize,
}

#[class_methods]
impl FontSizeObj {
    const fn new(size: FontSize) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            size,
        }
    }

    /// An extra-small font size with a value of one-fifth.
    #[constant]
    const EXTRA_SMALL: &Self = &Self::new(FontSize::EXTRA_SMALL);
    /// A small font size with a value of one-fourth.
    #[constant]
    const SMALL: &Self = &Self::new(FontSize::SMALL);
    /// A medium font size with a value of one-third.
    #[constant]
    const MEDIUM: &Self = &Self::new(FontSize::MEDIUM);
    /// A large font size with a value of one-half.
    #[constant]
    const LARGE: &Self = &Self::new(FontSize::LARGE);
    /// An extra-large font size with a value of two-thirds.
    #[constant]
    const EXTRA_LARGE: &Self = &Self::new(FontSize::EXTRA_LARGE);
    /// The full size of the font.
    #[constant]
    const FULL: &Self = &Self::new(FontSize::FULL);

    /// Creates a custom fractional font size from `numerator` and `denominator`.
    ///
    /// Both values must be nonnegative integers. `denominator` should be greater than zero, although the
    /// current runtime doesn't validate that constraint.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `numerator` or `denominator` is negative or outside the supported integer range.
    #[make_new]
    #[stub(sig = "(self, numerator: int, denominator: int) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);

        let numerator = reader.next_positional()?;
        let denominator = reader.next_positional()?;

        Ok(Self {
            base: ty.into(),
            size: FontSize {
                numerator,
                denominator,
            },
        })
    }

    #[attr]
    #[stub(attrs = ["numerator: int", "denominator: int"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "numerator" => self.size.numerator as i32,
            "denominator" => self.size.denominator as i32,
            _ => return,
        })
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "FontSize(numerator={}, denominator={})",
            self.size.numerator, self.size.denominator
        );
    }
}

/// Draws a filled pixel to the display with the specified `color`.
///
/// `x` and `y` are pixel coordinates. The writable display is 480 pixels wide by 240 pixels high, with
/// its origin at the top-left.
#[fun]
fn draw_pixel(x: i16, y: i16, color: &ColorObj) {
    lock_display().fill(&Point2 { x, y }, color.color());
}

/// Draws a line to the display with the specified `color`.
///
/// `start_x` and `start_y` are the start point of the line; `end_x` and `end_y` are the end point. The
/// line width is one pixel. Coordinates are measured from the display's top-left. The underlying
/// drawing implementation currently subtracts one from both supplied end coordinates before calling
/// the display SDK.
#[fun(ty = var_between(min = 5, max = 5))]
#[stub(sig = "(start_x: int, start_y: int, end_x: int, end_y: int, color: Color) -> None")]
fn draw_line(args: &[Obj]) -> Result<(), Exception> {
    let mut reader = Args::new(5, 0, args).reader();
    let start_x = reader.next_positional()?;
    let start_y = reader.next_positional()?;
    let end_x = reader.next_positional()?;
    let end_y = reader.next_positional()?;
    let color = reader.next_positional::<&ColorObj>()?;

    lock_display().fill(
        &Line::new(
            Point2 {
                x: start_x,
                y: start_y,
            },
            Point2 { x: end_x, y: end_y },
        ),
        color.color(),
    );
    Ok(())
}

fn parse_circle_args(args: &[Obj]) -> Result<(i16, i16, u16, &ColorObj), Exception> {
    let mut reader = Args::new(5, 0, args).reader();
    let x = reader.next_positional()?;
    let y = reader.next_positional()?;
    let radius = reader.next_positional()?;
    let color = reader.next_positional::<&ColorObj>()?;
    Ok((x, y, radius, color))
}

/// Draws a outlined circle to the display with the specified `color`.
///
/// `x` and `y` are the center point of the circle, and `radius` is its radius in pixels. Circles are
/// not antialiased.
#[fun(ty = var_between(min = 4, max = 4))]
#[stub(sig = "(x: int, y: int, radius: int, color: Color) -> None")]
fn draw_circle(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, radius, color) = parse_circle_args(args)?;
    lock_display().stroke(&Circle::new(Point2 { x, y }, radius), color.color());
    Ok(())
}

/// Draws a filled circle to the display with the specified `color`.
///
/// `x` and `y` are the center point of the circle, and `radius` is its radius in pixels. Circles are
/// not antialiased.
#[fun(ty = var_between(min = 4, max = 4))]
#[stub(sig = "(x: int, y: int, radius: int, color: Color) -> None")]
fn fill_circle(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, radius, color) = parse_circle_args(args)?;
    lock_display().fill(&Circle::new(Point2 { x, y }, radius), color.color());
    Ok(())
}

fn parse_rect_args(args: &[Obj]) -> Result<(i16, i16, u16, u16, &ColorObj), Exception> {
    let mut reader = Args::new(5, 0, args).reader();
    let x = reader.next_positional()?;
    let y = reader.next_positional()?;
    let width = reader.next_positional()?;
    let height = reader.next_positional()?;
    let color = reader.next_positional::<&ColorObj>()?;
    Ok((x, y, width, height, color))
}

/// Draws an outlined rectangular region of the display with the specified `color`.
///
/// `x` and `y` are the top-left coordinate of the rectangle. `width` and `height` are its dimensions
/// in pixels. The bottom right point is not included in the shape's bounds. Thus, the area of the
/// drawn rectangle is `width * height` pixels.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// # Draw a 20x20 rectangle which has a top-left point at (30, 40).
/// display.draw_rect(30, 40, 20, 20, Color.WHITE)
/// ```
#[fun(ty = var_between(min = 5, max = 5))]
#[stub(sig = "(x: int, y: int, width: int, height: int, color: Color) -> None")]
fn draw_rect(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, width, height, color) = parse_rect_args(args)?;
    lock_display().stroke(
        &Rect::from_dimensions(Point2 { x, y }, width, height),
        color.color(),
    );
    Ok(())
}

/// Draws a filled rectangular region of the display with the specified `color`.
///
/// `x` and `y` are the top-left coordinate of the rectangle. `width` and `height` are its dimensions
/// in pixels. The bottom right point is not included in the shape's bounds. Thus, the area of the
/// drawn rectangle is `width * height` pixels.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// # Draw a 20x20 rectangle which has a top-left point at (30, 40).
/// display.fill_rect(30, 40, 20, 20, Color.WHITE)
/// ```
#[fun(ty = var_between(min = 5, max = 5))]
#[stub(sig = "(x: int, y: int, width: int, height: int, color: Color) -> None")]
fn fill_rect(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, width, height, color) = parse_rect_args(args)?;
    lock_display().fill(
        &Rect::from_dimensions(Point2 { x, y }, width, height),
        color.color(),
    );
    Ok(())
}

/// Draws a buffer of pixels to a specified region of the display.
///
/// This function copies the pixels in `buffer` to the specified region of the display. `x` and `y`
/// are the region's top-left corner, and `width` and `height` are measured in pixels. `buffer` must be
/// a readable, suitably aligned buffer containing exactly `width * height` packed four-byte
/// `0xRRGGBB` color values in row-major order. The current implementation uses an internal assertion
/// for an incorrect pixel count instead of translating it to a Python exception.
///
/// # Raises
///
/// - `TypeError`: If `buffer` doesn't support the readable buffer protocol.
/// - `ValueError`: If the readable byte length isn't a multiple of four.
#[fun(ty = var_between(min = 5, max = 5))]
#[stub(sig = "(x: int, y: int, width: int, height: int, buffer: Any) -> None")]
fn draw_buffer(args: &[Obj]) -> Result<(), Exception> {
    let mut reader = Args::new(5, 0, args).reader();
    let x = reader.next_positional()?;
    let y = reader.next_positional()?;
    let width = reader.next_positional()?;
    let height = reader.next_positional()?;
    let buffer = reader.next_positional::<Buffer<'_, Color>>()?;

    lock_display().draw_buffer(
        Rect::from_dimensions(Point2 { x, y }, width, height),
        buffer.buffer(),
    );
    Ok(())
}

/// Draws a line of `text` with the specified `color` and `bg_color` to the display.
///
/// `x` and `y` are the top-left corner coordinates of the text. `font_size` defaults to
/// `FontSize.MEDIUM`, `font_family` defaults to `FontFamily.PROPORTIONAL`, and `color` defaults to
/// `Color.WHITE`. Omitting `bg_color` gives a transparent background; supply a `Color` to paint it.
/// Although the signature accepts `None`, the current implementation rejects an explicitly supplied
/// `bg_color=None`, so omit the keyword instead.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// # Write red text with a blue background to the display.
/// display.draw_text(
///     "Hello, World!",
///     10,
///     10,
///     font_size=display.FontSize.MEDIUM,
///     font_family=display.FontFamily.MONOSPACE,
///     color=Color(255, 0, 0),
///     bg_color=Color(0, 0, 255),
/// )
/// ```
///
/// # Raises
///
/// - `ValueError`: If `text` contains a NUL character.
#[fun(ty = kw(min = 3))]
#[stub(
    sig = "(text: str, x: int, y: int, *, font_size: FontSize = FontSize.MEDIUM, font_family: FontFamily = FontFamily.PROPORTIONAL, color: Color = Color.WHITE, bg_color: Color | None = None) -> None"
)]
fn draw_text(args: &[Obj], kw_map: &Map) -> Result<(), Exception> {
    let kwarg_count = kw_map.len();
    let mut reader = Args::new(args.len(), kwarg_count, args).reader();
    reader.assert_npos(3, 3).assert_nkw(0, 4);

    let cstr = reader.next_positional()?;
    let x = reader.next_positional()?;
    let y = reader.next_positional()?;

    let mut font_size = FontSize::MEDIUM;
    let mut font_family = FontFamily::Proportional;
    let mut color = Color::WHITE;
    let mut bg_color = None;

    while let Some(arg) = reader.next_kw() {
        match arg.kw {
            "font_size" => font_size = arg.parse::<&FontSizeObj>()?.size,
            "font_family" => font_family = arg.parse::<&FontFamilyObj>()?.family,
            "color" => color = arg.parse::<&ColorObj>()?.color(),
            "bg_color" => bg_color = Some(arg.parse::<&ColorObj>()?.color()),
            _ => Err(type_error(error_msg!("unknown argument '{}'", arg.kw)))?,
        }
    }

    let font = Font::new(font_size, font_family);
    let text = Text::new(cstr, font, Point2 { x, y });

    lock_display().draw_text(&text, color, bg_color);
    Ok(())
}

/// Writes `values` to the Brain display's scrolling text area.
///
/// Values are converted to strings and joined with `sep`, which defaults to one space, then `end` is
/// appended, which defaults to a newline. Text uses the default white font, wraps after 48 characters,
/// and scrolls upward after 12 visible lines.
///
/// # Raises
///
/// - `TypeError`: If a value isn't printable or if `sep` or `end` isn't a string.
#[fun(ty = kw(min = 0))]
#[stub(sig = "(*values: object, sep: str = ' ', end: str = '\\n') -> None")]
fn print(args: &[Obj], kw_map: &Map) -> Result<(), Exception> {
    let kwarg_count = kw_map.len();
    let mut reader = Args::new(args.len(), kwarg_count, args).reader();

    let mut sep = " ";
    let mut end = "\n";

    while let Some(arg) = reader.next_kw() {
        match arg.kw {
            "sep" => sep = arg.parse()?,
            "end" => end = arg.parse()?,
            _ => Err(type_error(error_msg!("unknown argument '{}'", arg.kw)))?,
        }
    }

    let mut string = String::new();
    let mut string_print = StringPrint::new(&mut string);

    let mut first = true;
    loop {
        let arg = match reader.next_positional::<Obj>() {
            Ok(v) => v,
            Err(e) => match e {
                PositionalError::ArgumentsExhausted => break,
                _ => Err(e)?,
            },
        };

        if !first {
            string_print.string().push_str(sep);
        }
        first = false;

        arg.print(string_print.print(), PrintKind::Str)
            .map_err(|_| {
                type_error(error_msg!(
                    "type '{}' is not printable",
                    arg.obj_type().name().as_str()
                ))
            })?;
    }

    string.push_str(end);
    lock_display().write_str(&string).unwrap(); // function is infallible

    Ok(())
}

/// Scrolls the pixels at or below the specified y-coordinate `start`.
///
/// This function y-offsets the pixels in the display buffer which are at or below `start` by `offset`
/// pixels. Positive values move the pixels upwards, and pixels that are moved out of the scroll region
/// are discarded. Empty spaces are then filled with the display's background color.
#[fun]
fn scroll(start: i16, offset: i16) {
    lock_display().scroll(start, offset);
}

/// Scrolls a region of the display.
///
/// This function y-offsets the pixels in the display buffer which are contained in the specified
/// scroll region by `offset` pixels. `x` and `y` are the region's top-left corner; `width` and `height`
/// are measured in pixels. Positive offset values move the pixels upwards, and pixels that are moved
/// out of the scroll region are discarded. Empty spaces are then filled with the display's background
/// color.
#[fun(ty = var_between(min = 5, max = 5))]
#[stub(sig = "(x: int, y: int, width: int, height: int, offset: int) -> None")]
fn scroll_region(args: &[Obj]) -> Result<(), Exception> {
    let mut reader = Args::new(5, 0, args).reader();
    let x = reader.next_positional()?;
    let y = reader.next_positional()?;
    let width = reader.next_positional()?;
    let height = reader.next_positional()?;
    let offset = reader.next_positional()?;

    lock_display().scroll_region(
        Rect::from_dimensions(Point2 { x, y }, width, height),
        offset,
    );
    Ok(())
}

/// Sets `render_mode` for the display.
///
/// For more information on render modes, see `display.RenderMode`.
#[fun]
fn set_render_mode(render_mode: &RenderModeObj) {
    lock_display().set_render_mode(render_mode.mode);
}

/// Flushes the display's double buffer if it is enabled.
///
/// This is a no-op with `RenderMode.IMMEDIATE`, but is necessary for anything to be displayed when
/// using `RenderMode.DOUBLE_BUFFERED`.
#[fun]
fn render() {
    lock_display().render();
}

/// Clears the entire 480-by-240-pixel writable display, filling it with the specified `color`.
#[fun]
fn erase(color: &ColorObj) {
    lock_display().erase(color.color());
}

/// A read-only touch event on the display, available as `display.TouchEvent`.
///
/// - `x` and `y` are the point at which the display was touched, in pixels from the top-left.
/// - `press_count` is the number of times the display has been pressed.
/// - `release_count` is the number of times the display has been released.
/// - `is_now_pressed` is `True` if the touch has just been pressed.
/// - `is_pressed` is `True` if the display has been touched or is still being held.
/// - `is_released` is `True` if the touch has been released.
/// - `is_held` is `True` if the display has been touched and is still being held.
///
/// Snapshots compare by value and are obtained from `display.get_touch_status`, not constructed
/// directly.
#[class(qstr!(TouchEvent))]
#[repr(C)]
struct TouchEventObj {
    base: ObjBase,
    event: TouchEvent,
}

#[class_methods]
impl TouchEventObj {
    #[attr]
    #[stub(attrs = [
        "x: int",
        "y: int",
        "press_count: int",
        "release_count: int",
        "is_now_pressed: bool",
        "is_pressed: bool",
        "is_released: bool",
        "is_held: bool",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "x" => Obj::from(self.event.point.x as i32),
            "y" => Obj::from(self.event.point.y as i32),

            "press_count" => Obj::from(self.event.press_count),
            "release_count" => Obj::from(self.event.release_count),

            "is_now_pressed" => Obj::from(self.event.state == TouchState::Pressed),
            "is_pressed" => Obj::from(matches!(
                self.event.state,
                TouchState::Pressed | TouchState::Held
            )),
            "is_released" => Obj::from(self.event.state == TouchState::Released),
            "is_held" => Obj::from(self.event.state == TouchState::Held),

            _ => return,
        });
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.event == rhs.as_obj::<Self>().event),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "TouchEvent(x={}, y={}, press_count={}, release_count={}, is_now_pressed={}, is_pressed={}, is_released={}, is_held={})",
            self.event.point.x,
            self.event.point.y,
            self.event.press_count,
            self.event.release_count,
            if self.event.state == TouchState::Pressed {
                "True"
            } else {
                "False"
            },
            if matches!(self.event.state, TouchState::Pressed | TouchState::Held) {
                "True"
            } else {
                "False"
            },
            if self.event.state == TouchState::Released {
                "True"
            } else {
                "False"
            },
            if self.event.state == TouchState::Held {
                "True"
            } else {
                "False"
            }
        );
    }
}

/// Returns the last recorded state of the display's touchscreen as a `TouchEvent`.
///
/// See `display.TouchEvent` for more information.
#[fun]
fn get_touch_status() -> TouchEventObj {
    TouchEventObj {
        base: TouchEventObj::OBJ_TYPE.into(),
        event: lock_display().touch_status(),
    }
}

/// Returns whether the touchscreen's last recorded touch has just been pressed.
#[fun]
fn is_now_pressed() -> bool {
    lock_display().touch_status().state == TouchState::Pressed
}

/// Returns whether the touchscreen's last recorded touch has been pressed or is being held.
#[fun]
fn is_pressed() -> bool {
    matches!(
        lock_display().touch_status().state,
        TouchState::Pressed | TouchState::Held
    )
}

/// Returns whether the touchscreen's last recorded touch has been released.
#[fun]
fn is_released() -> bool {
    lock_display().touch_status().state == TouchState::Released
}

/// Returns whether the touchscreen's last recorded touch is still being held.
#[fun]
fn is_held() -> bool {
    lock_display().touch_status().state == TouchState::Held
}
