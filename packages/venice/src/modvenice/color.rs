use std::fmt::Write;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    self,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::color::Color;

use crate::modvenice::{Exception, read_only_attr::read_only_attr};

/// A color stored in RGB format for devices and graphics.
///
/// The read-only `r`, `g`, and `b` attributes are the red, green, and blue channels in the inclusive
/// range 0 to 255. Colors compare equal when all three channels match, and their readable
/// representation is `Color(r=..., g=..., b=...)`.
#[class(qstr!(Color))]
#[repr(C)]
pub struct ColorObj {
    base: ObjBase,
    color: Color,
}

#[class_methods]
impl ColorObj {
    const fn new(color: Color) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            color,
        }
    }

    /// "White" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const WHITE: &Self = &Self::new(Color::WHITE);
    /// "Silver" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const SILVER: &Self = &Self::new(Color::SILVER);
    /// "Gray" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const GRAY: &Self = &Self::new(Color::GRAY);
    /// "Black" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const BLACK: &Self = &Self::new(Color::BLACK);
    /// "Red" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const RED: &Self = &Self::new(Color::RED);
    /// "Maroon" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const MAROON: &Self = &Self::new(Color::MAROON);
    /// "Yellow" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const YELLOW: &Self = &Self::new(Color::YELLOW);
    /// "Olive" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const OLIVE: &Self = &Self::new(Color::OLIVE);
    /// "Lime" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const LIME: &Self = &Self::new(Color::LIME);
    /// "Green" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const GREEN: &Self = &Self::new(Color::GREEN);
    /// "Aqua" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const AQUA: &Self = &Self::new(Color::AQUA);
    /// "Teal" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const TEAL: &Self = &Self::new(Color::TEAL);
    /// "Blue" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const BLUE: &Self = &Self::new(Color::BLUE);
    /// "Navy" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const NAVY: &Self = &Self::new(Color::NAVY);
    /// "Fuchsia" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const FUCHSIA: &Self = &Self::new(Color::FUCHSIA);
    /// "Purple" color as defined in the HTML 4.01 specification.
    #[constant]
    pub const PURPLE: &Self = &Self::new(Color::PURPLE);

    /// Creates a new RGB color from the provided components `r`, `g`, and `b`.
    ///
    /// Each channel is a positional-only integer from 0 to 255 and defaults to 0, so `Color()` is black.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// orange = Color(255, 128, 0)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a channel is not an integer, a keyword argument is supplied, or too many
    ///   positional arguments are given.
    /// - `ValueError`: If a channel is outside the inclusive range 0 to 255.
    #[make_new]
    #[stub(sig = "(self, r: int = 0, g: int = 0, b: int = 0) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 3).assert_nkw(0, 0);

        let r = reader.next_positional_or(0)?;
        let g = reader.next_positional_or(0)?;
        let b = reader.next_positional_or(0)?;

        Ok(Self {
            base: ty.into(),
            color: Color::new(r, g, b),
        })
    }

    #[attr]
    #[stub(attrs = ["r: int", "g: int", "b: int"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "r" => self.color.r as i32,
            "g" => self.color.g as i32,
            "b" => self.color.b as i32,
            _ => return,
        });
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// Converts this color to a raw `0xRRGGBB` representation.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// assert Color(255, 128, 0).as_int() == 0xFF8000
    /// ```
    #[method]
    fn as_int(&self) -> i32 {
        self.color.into_raw() as i32
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.color == rhs.as_obj::<Self>().color),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "Color(r={}, g={}, b={})",
            self.color.r, self.color.g, self.color.b
        );
    }
}
