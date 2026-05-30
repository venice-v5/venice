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
