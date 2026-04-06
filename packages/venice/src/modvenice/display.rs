use std::fmt::Write;

use argparse::{Args, PositionalError, error_msg};
use micropython_macros::{class, class_methods, fun};
use micropython_rs::{
    buffer::Buffer,
    const_dict,
    except::type_error,
    map::{Dict, Map},
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    print::{PrintKind, StringPrint},
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
    modvenice::{Exception, color::ColorObj},
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

    #[constant]
    const IMMEDIATE: &Self = &Self::new(RenderMode::Immediate);
    #[constant]
    const DOUBLE_BUFFERED: &Self = &Self::new(RenderMode::DoubleBuffered);
}

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

    #[constant]
    const MONOSPACE: &Self = &Self::new(FontFamily::Monospace);
    #[constant]
    const PROPORTIONAL: &Self = &Self::new(FontFamily::Proportional);
}

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

    #[constant]
    const EXTRA_SMALL: &Self = &Self::new(FontSize::EXTRA_SMALL);
    #[constant]
    const SMALL: &Self = &Self::new(FontSize::SMALL);
    #[constant]
    const MEDIUM: &Self = &Self::new(FontSize::MEDIUM);
    #[constant]
    const LARGE: &Self = &Self::new(FontSize::LARGE);
    #[constant]
    const EXTRA_LARGE: &Self = &Self::new(FontSize::EXTRA_LARGE);
    #[constant]
    const FULL: &Self = &Self::new(FontSize::FULL);

    #[make_new]
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
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "numerator" => self.size.numerator as i32,
            "denominator" => self.size.denominator as i32,
            _ => return,
        })
    }
}

#[fun]
fn draw_pixel(x: i16, y: i16, color: &ColorObj) {
    lock_display().fill(&Point2 { x, y }, color.color());
}

#[fun(ty = var_between(min = 5, max = 5))]
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

#[fun(ty = var_between(min = 4, max = 4))]
fn draw_circle(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, radius, color) = parse_circle_args(args)?;
    lock_display().stroke(&Circle::new(Point2 { x, y }, radius), color.color());
    Ok(())
}

#[fun(ty = var_between(min = 4, max = 4))]
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

#[fun(ty = var_between(min = 5, max = 5))]
fn draw_rect(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, width, height, color) = parse_rect_args(args)?;
    lock_display().stroke(
        &Rect::from_dimensions(Point2 { x, y }, width, height),
        color.color(),
    );
    Ok(())
}

#[fun(ty = var_between(min = 5, max = 5))]
fn fill_rect(args: &[Obj]) -> Result<(), Exception> {
    let (x, y, width, height, color) = parse_rect_args(args)?;
    lock_display().fill(
        &Rect::from_dimensions(Point2 { x, y }, width, height),
        color.color(),
    );
    Ok(())
}

#[fun(ty = var_between(min = 5, max = 5))]
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

#[fun(ty = kw(min = 3))]
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

#[fun(ty = kw(min = 0))]
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

        arg.print(string_print.print(), PrintKind::Str)
            .map_err(|_| {
                type_error(error_msg!(
                    "type '{}' is not printable",
                    arg.obj_type().name().as_str()
                ))
            })?;

        if !first {
            string_print.string().push_str(sep);
        }
        first = false;
    }

    string.push_str(end);
    lock_display().write_str(&string).unwrap(); // function is infallible

    Ok(())
}

#[fun]
fn scroll(start: i16, offset: i16) {
    lock_display().scroll(start, offset);
}

#[fun(ty = var_between(min = 5, max = 5))]
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

#[fun]
fn set_render_mode(render_mode: &RenderModeObj) {
    lock_display().set_render_mode(render_mode.mode);
}

#[fun]
fn render() {
    lock_display().render();
}

#[fun]
fn erase(color: &ColorObj) {
    lock_display().erase(color.color());
}

#[class(qstr!(TouchEvent))]
#[repr(C)]
struct TouchEventObj {
    base: ObjBase,
    event: TouchEvent,
}

#[class_methods]
impl TouchEventObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
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
}

#[fun]
fn get_touch_status() -> TouchEventObj {
    TouchEventObj {
        base: TouchEventObj::OBJ_TYPE.into(),
        event: lock_display().touch_status(),
    }
}

#[fun]
fn is_now_pressed() -> bool {
    lock_display().touch_status().state == TouchState::Pressed
}

#[fun]
fn is_pressed() -> bool {
    matches!(
        lock_display().touch_status().state,
        TouchState::Pressed | TouchState::Held
    )
}

#[fun]
fn is_released() -> bool {
    lock_display().touch_status().state == TouchState::Released
}

#[fun]
fn is_held() -> bool {
    lock_display().touch_status().state == TouchState::Held
}
