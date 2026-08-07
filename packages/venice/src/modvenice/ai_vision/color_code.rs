use std::{cell::Cell, fmt::Write};

use argparse::{Args, PositionalError};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjTrait, ObjType, SubscrOp},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
};
use vexide_devices::smart::ai_vision::AiVisionColorCode;

use crate::modvenice::Exception;

/// A color code used by an AI Vision Sensor to detect groups of color blobs.
///
/// Color codes are effectively "groups" of color signatures. A color code associated with multiple
/// color signatures on the sensor will be detected as a single object when all signatures are seen
/// next to each other.
///
/// Color codes can associate up to 7 color signatures and detections will be returned as
/// `AiVisionCodeObject` instances.
///
/// Indexing uses positions 0 through 6; reading returns an `int` or `None`, and assignment accepts an
/// `int` or `None`. Deleting an item isn't supported. Codes compare by value. Sensor color-signature
/// IDs are 1 through 7. The current implementation doesn't bounds-check subscript indices, so an
/// index outside 0 through 6 can terminate the operation instead of raising a Python exception.
#[class(qstr!(AiVisionColorCode))]
#[repr(C)]
pub struct AiVisionColorCodeObj {
    base: ObjBase,
    // this is the backing type for AiVisionColorCode
    // we store it this way to make mutability easier
    code: Cell<[Option<u8>; 7]>,
}

impl AiVisionColorCodeObj {
    pub fn code(&self) -> AiVisionColorCode {
        // WHAT DOES HE EVEN DO?
        AiVisionColorCode::new::<7>(self.code.get())
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
    /// `color1` is required; `color2`, `color3`, `color4`, `color5`, `color6`, and `color7` may be omitted.
    /// Each supplied value must fit in an unsigned byte; use IDs from 1 through 7 when the code will be
    /// registered with an `AiVisionSensor`.
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
    /// - `ValueError`: If a supplied color ID is outside 0 through 255.
    #[make_new]
    #[stub(
        sig = "(self, color1: int, color2: int | None = None, color3: int | None = None, color4: int | None = None, color5: int | None = None, color6: int | None = None, color7: int | None = None) -> None"
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
            let res = reader.next_positional::<u8>();
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
        match op {
            SubscrOp::Delete => Obj::NULL,
            SubscrOp::Store { src } => {
                let value = if let Some(v) = src.try_to_int() {
                    Some(v as u8)
                } else if src.is_none() {
                    None
                } else {
                    return Obj::NULL;
                };
                let mut code = self.code.get();
                code[index as usize] = value;
                self.code.set(code);
                Obj::NONE
            }
            SubscrOp::Load => {
                if let Some(v) = self.code.get()[index as usize] {
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
            BinaryOpCode::Equal => {
                Obj::from_bool(lhs.code.get() == rhs.as_obj::<Self>().code.get())
            }
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let code = self.code.get();
        let _ = write!(print, "AiVisionColorCode(color1={}", code[0].unwrap());

        for (i, value) in code.iter().enumerate().skip(1) {
            if let Some(value) = value {
                let _ = write!(print, ", color{}={value}", i + 1);
            }
        }

        print.print(")");
    }
}
