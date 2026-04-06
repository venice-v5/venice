use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    self,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::color::Color;

use crate::modvenice::{Exception, read_only_attr::read_only_attr};

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

    #[constant]
    pub const WHITE: &Self = &Self::new(Color::WHITE);
    #[constant]
    pub const SILVER: &Self = &Self::new(Color::SILVER);
    #[constant]
    pub const GRAY: &Self = &Self::new(Color::GRAY);
    #[constant]
    pub const BLACK: &Self = &Self::new(Color::BLACK);
    #[constant]
    pub const RED: &Self = &Self::new(Color::RED);
    #[constant]
    pub const MAROON: &Self = &Self::new(Color::MAROON);
    #[constant]
    pub const YELLOW: &Self = &Self::new(Color::YELLOW);
    #[constant]
    pub const OLIVE: &Self = &Self::new(Color::OLIVE);
    #[constant]
    pub const LIME: &Self = &Self::new(Color::LIME);
    #[constant]
    pub const GREEN: &Self = &Self::new(Color::GREEN);
    #[constant]
    pub const AQUA: &Self = &Self::new(Color::AQUA);
    #[constant]
    pub const TEAL: &Self = &Self::new(Color::TEAL);
    #[constant]
    pub const BLUE: &Self = &Self::new(Color::BLUE);
    #[constant]
    pub const NAVY: &Self = &Self::new(Color::NAVY);
    #[constant]
    pub const FUCHSIA: &Self = &Self::new(Color::FUCHSIA);
    #[constant]
    pub const PURPLE: &Self = &Self::new(Color::PURPLE);

    #[make_new]
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

    #[method]
    fn as_int(&self) -> i32 {
        self.color.into_raw() as i32
    }
}
