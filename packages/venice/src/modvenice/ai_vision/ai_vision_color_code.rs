use std::cell::Cell;

use argparse::{Args, PositionalError};
use micropython_rs::{
    class, class_methods,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType, SubscrOp},
};
use vexide_devices::smart::ai_vision::AiVisionColorCode;

use crate::modvenice::Exception;

#[class(qstr!(AiVisionColorCode))]
#[repr(C)]
pub struct AiVisionColorCodeObj {
    base: ObjBase<'static>,
    // this is the backing type for AiVisionColorCode
    // we store it this way to make mutability easier
    code: Cell<[Option<u8>; 7]>,
}

impl AiVisionColorCodeObj {
    pub fn code(&self) -> AiVisionColorCode {
        AiVisionColorCode::new::<7>(self.code.get())
    }

    pub fn new(color: AiVisionColorCode) -> Self {
        let mut codes = [None; 7];
        for (c, code) in color.iter().zip(codes.iter_mut()) {
            *code = Some(c);
        }
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            code: Cell::new(codes),
        }
    }
}

#[class_methods]
impl AiVisionColorCodeObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());
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
}
