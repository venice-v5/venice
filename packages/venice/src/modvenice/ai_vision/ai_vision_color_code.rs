use std::cell::Cell;

use micropython_rs::{
    init::token,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, SubscrOp, TypeFlags},
    subscr_from_fn,
};
use vexide_devices::smart::ai_vision::AiVisionColorCode;

use crate::{args::Args, obj::alloc_obj, qstrgen::qstr};

static AI_VISION_COLOR_CODE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionColorCode))
        .set_make_new(make_new_from_fn!(ai_vision_color_code_make_new))
        .set_subscr(subscr_from_fn!(ai_vision_color_code_subscr));

#[repr(C)]
pub struct AiVisionColorCodeObj {
    base: ObjBase<'static>,
    // this is the backing type for AiVisionColorCode
    // we store it this way to make mutability easier
    code: Cell<[Option<u8>; 7]>,
}

unsafe impl ObjTrait for AiVisionColorCodeObj {
    const OBJ_TYPE: &ObjType = AI_VISION_COLOR_CODE_OBJ_TYPE.as_obj_type();
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
            base: ObjBase::new(AI_VISION_COLOR_CODE_OBJ_TYPE.as_obj_type()),
            code: Cell::new(codes),
        }
    }
}

fn ai_vision_color_code_make_new(
    ty: &'static ObjType,
    n_pos: usize,
    n_kw: usize,
    args: &[Obj],
) -> Obj {
    let mut reader = Args::new(n_pos, n_kw, args).reader(token().unwrap());
    reader.assert_npos(1, 7);
    let mut values = [None; 7];
    for value in values.iter_mut() {
        let res = reader.try_next_positional::<i32>();
        if let Ok(v) = res {
            // TODO: this conversion needs a bounds check
            *value = Some(v as u8);
        } else {
            break;
        }
    }
    alloc_obj(AiVisionColorCodeObj {
        base: ObjBase::new(ty),
        code: Cell::new(values),
    })
}

fn ai_vision_color_code_subscr(this: &AiVisionColorCodeObj, index: i32, op: SubscrOp) -> Obj {
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
            let mut code = this.code.get();
            code[index as usize] = value;
            this.code.set(code);
            Obj::NONE
        }
        SubscrOp::Load => {
            if let Some(v) = this.code.get()[index as usize] {
                Obj::from_int(v as _)
            } else {
                Obj::NONE
            }
        }
    }
}
