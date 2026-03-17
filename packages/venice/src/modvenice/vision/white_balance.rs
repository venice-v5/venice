use argparse::Args;
use micropython_rs::{
    class, class_methods,
    except::raise_type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::smart::vision::WhiteBalance;

use crate::modvenice::Exception;

#[class(qstr!(WhiteBalance))]
#[repr(C)]
pub struct WhiteBalanceObj {
    base: ObjBase<'static>,
}

#[class(qstr!(Auto))]
#[repr(C)]
pub struct Auto {
    base: ObjBase<'static>,
}

#[class(qstr!(StartupAuto))]
#[repr(C)]
pub struct StartupAuto {
    base: ObjBase<'static>,
}

#[class(qstr!(Manual))]
#[repr(C)]
pub struct Manual {
    base: ObjBase<'static>,
    r: u8,
    g: u8,
    b: u8,
}

#[class_methods]
impl WhiteBalanceObj {
    #[make_new]
    fn make_new(_: &ObjType, _: usize, _: usize, _: &[Obj]) {
        raise_type_error(
            token(),
            c"WhiteBalance is an abstract base class; use a variant like WhiteBalance.Signature",
        );
    }

    #[constant(qstr!(Auto))]
    const AUTO: &ObjType = Auto::OBJ_TYPE;
    #[constant(qstr!(StartupAuto))]
    const STARTUP_AUTO: &ObjType = StartupAuto::OBJ_TYPE;
    #[constant(qstr!(Manual))]
    const MANUAL: &ObjType = Manual::OBJ_TYPE;
}

#[class_methods]
impl Auto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, _: &[Obj]) -> Obj {
        if n_pos != 0 || n_kw != 0 {
            raise_type_error(
                token(),
                c"constructor does not accept arguments; just call WhiteBalance.Auto()",
            );
        }

        Obj::from_static(Self::SELF)
    }
}

#[class_methods]
impl StartupAuto {
    pub const SELF: &Self = &Self {
        base: ObjBase::new(Self::OBJ_TYPE),
    };

    #[make_new]
    fn make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, _: &[Obj]) -> Obj {
        if n_pos != 0 || n_kw != 0 {
            raise_type_error(
                token(),
                c"constructor does not accept arguments; just call WhiteBalance.StartupAuto()",
            );
        }

        Obj::from_static(Self::SELF)
    }
}

#[class_methods]
impl Manual {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());
        reader.assert_npos(3, 3).assert_nkw(0, 0);

        let r = reader.next_positional::<u8>()?;
        let g = reader.next_positional::<u8>()?;
        let b = reader.next_positional::<u8>()?;

        Ok(Self {
            base: ObjBase::new(ty),
            r,
            g,
            b,
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "r" => self.r,
            "g" => self.g,
            "b" => self.b,
            _ => return,
        } as i32)
    }
}

pub fn new(balance: WhiteBalance) -> Obj {
    match balance {
        WhiteBalance::Auto => Obj::from_static(Auto::SELF),
        WhiteBalance::StartupAuto => Obj::from_static(StartupAuto::SELF),
        WhiteBalance::Manual(color) => Manual {
            base: ObjBase::new(Manual::OBJ_TYPE),
            r: color.r,
            g: color.g,
            b: color.b,
        }
        .into(),
    }
}

pub fn from_obj(obj: Obj) -> Option<WhiteBalance> {
    if obj.is(Auto::OBJ_TYPE) {
        Some(WhiteBalance::Auto)
    } else if obj.is(StartupAuto::OBJ_TYPE) {
        Some(WhiteBalance::StartupAuto)
    } else if let Some(manual) = obj.try_as_obj::<Manual>() {
        Some(WhiteBalance::Manual((manual.r, manual.g, manual.b).into()))
    } else {
        None
    }
}
