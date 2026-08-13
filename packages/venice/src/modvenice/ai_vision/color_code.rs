use std::{cell::Cell, fmt::Write};

use argparse::{Args, IntParser, PositionalError, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{index_error, type_error, value_error},
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType, SubscrOp},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
};
use vexide_devices::smart::ai_vision::AiVisionColorCode;

use crate::modvenice::{Exception, ai_vision::validation::narrow_slot_id};

/// A color code used by an AI Vision Sensor to detect groups of color blobs.
///
/// Color codes are effectively "groups" of color signatures. A color code associated with multiple
/// color signatures on the sensor will be detected as a single object when all signatures are seen
/// next to each other.
///
/// Color codes can associate up to 7 color signatures and detections will be returned as
/// `AiVisionCodeObject` instances.
///
/// Indexing uses positions 0 through 6; reading returns an `int` or `None`, and assignment accepts a
/// color-signature ID from 1 through 7 or `None`. Deleting an item isn't supported. Codes compare by
/// value and return `False` when compared with another type. An out-of-range index raises
/// `IndexError`; assigning an out-of-range ID raises `ValueError`, and assigning another type raises
/// `TypeError`. Iteration visits all seven indexed positions, including `None` entries. The readable
/// representation lists each populated slot as `colorN=value`.
#[class(qstr!(AiVisionColorCode))]
#[repr(C)]
pub struct AiVisionColorCodeObj {
    base: ObjBase,
    // this is the backing type for AiVisionColorCode
    // we store it this way to make mutability easier
    code: Cell<[Option<u8>; 7]>,
}

impl AiVisionColorCodeObj {
    pub fn values(&self) -> [Option<u8>; 7] {
        self.code.get()
    }

    pub fn new(color: AiVisionColorCode) -> Self {
        let mut codes = [None; 7];
        for (c, code) in color.iter().zip(codes.iter_mut()) {
            *code = Some(c);
        }
        Self {
            base: Self::OBJ_TYPE.into(),
            code: Cell::new(codes),
        }
    }
}

// TODO: refactor this API to be more practical for competition use
#[class_methods]
impl AiVisionColorCodeObj {
    /// Creates a new color code with the given color signature IDs.
    ///
    /// `color1` is required; `color2`, `color3`, `color4`, `color5`, `color6`, and `color7` may be
    /// omitted rather than passed as `None`. Supply between one and seven positional-only integer
    /// IDs, each from 1 through 7.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// code = AiVisionColorCode(1, 2)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a supplied ID isn't an integer, `None` is passed explicitly, a keyword
    ///   argument is supplied, or the argument count is outside one to seven.
    /// - `ValueError`: If a supplied color ID is outside 1 through 7.
    #[make_new]
    #[stub(sig = "(self, color1: int, /) -> None")]
    #[stub(sig = "(self, color1: int, color2: int, /) -> None")]
    #[stub(sig = "(self, color1: int, color2: int, color3: int, /) -> None")]
    #[stub(sig = "(self, color1: int, color2: int, color3: int, color4: int, /) -> None")]
    #[stub(
        sig = "(self, color1: int, color2: int, color3: int, color4: int, color5: int, /) -> None"
    )]
    #[stub(
        sig = "(self, color1: int, color2: int, color3: int, color4: int, color5: int, color6: int, /) -> None"
    )]
    #[stub(
        sig = "(self, color1: int, color2: int, color3: int, color4: int, color5: int, color6: int, color7: int, /) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 7).assert_nkw(0, 0);

        let mut values = [None; 7];
        for value in values.iter_mut() {
            let res = reader.next_positional_with(IntParser::new(1..=7));
            match res {
                Ok(v) => *value = Some(v),
                Err(e) => match e {
                    PositionalError::ArgumentsExhausted => break,
                    _ => return Err(e.into()),
                },
            }
        }

        Ok(Self {
            base: ObjBase::new(ty),
            code: Cell::new(values),
        })
    }

    #[subscr]
    fn subcr(&self, index: i32, op: SubscrOp) -> Obj {
        let Some(index) = usize::try_from(index).ok().filter(|index| *index < 7) else {
            index_error(error_msg!(
                "color code index ({index}) is outside 0 through 6"
            ))
            .raise(token());
        };

        match op {
            SubscrOp::Delete => Obj::NULL,
            SubscrOp::Store { src } => {
                let value = if let Some(value) = src.try_to_int() {
                    Some(narrow_slot_id(value, 7).unwrap_or_else(|| {
                        value_error(error_msg!(
                            "color signature ID ({value}) is outside 1 through 7"
                        ))
                        .raise(token())
                    }))
                } else if src.is_none() {
                    None
                } else {
                    type_error(c"color code entries must be an int or None").raise(token());
                };
                let mut code = self.code.get();
                code[index] = value;
                self.code.set(code);
                Obj::NONE
            }
            SubscrOp::Load => {
                if let Some(v) = self.code.get()[index] {
                    Obj::from_int(v as _)
                } else {
                    Obj::NONE
                }
            }
        }
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(
                rhs.try_as_obj::<Self>()
                    .is_some_and(|rhs| lhs.code.get() == rhs.code.get()),
            ),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("AiVisionColorCode(");
        let mut first = true;
        for (index, value) in self.code.get().into_iter().enumerate() {
            if let Some(value) = value {
                if !first {
                    print.print(", ");
                }
                let _ = write!(print, "color{}={value}", index + 1);
                first = false;
            }
        }
        print.print(")");
    }
}
