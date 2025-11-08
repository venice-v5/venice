use std::fmt::Display;

use micropython_rs::{
    except::raise_type_error,
    init::InitToken,
    obj::{Obj, ObjTrait, ObjType, repr_c},
    str::Str,
};

#[derive(Clone, Copy)]
pub struct Args<'a> {
    n_pos: usize,
    n_kw: usize,
    args: &'a [Obj],
}

#[derive(Clone, Copy)]
pub struct ArgsReader<'a> {
    args: Args<'a>,
    n: usize,
    token: InitToken,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgType {
    Int,
    Str,
    None,
    Bool,
    Float,
    Obj(&'static ObjType),
}

#[derive(Clone, Copy)]
pub enum ArgValue<'a> {
    Int(i32),
    Str(&'a [u8]),
    None,
    Bool(bool),
    Float(f32),
    Obj(Obj),
}

#[derive(Clone, Copy)]
pub struct KwArg<'a> {
    pub kw: &'a [u8],
    pub value: ArgValue<'a>,
}

#[derive(Clone, Copy)]
pub enum Arg<'a> {
    Positional(ArgValue<'a>),
    Keyword(KwArg<'a>),
}

#[derive(Debug)]
pub enum ArgError {
    NotPresent,
    TypeMismatch {
        n: usize,
        expected: ArgType,
        found: ArgType,
    },
    PositionalsExhuasted {
        n: usize,
    },
    ExpectedKeyword,
    KeywordsExhuasted,
}

impl ArgType {
    pub fn of(obj: &Obj) -> Self {
        use repr_c::Ty;
        match obj.ty().unwrap() {
            Ty::Int => Self::Int,
            Ty::Qstr => Self::Str,
            Ty::Immediate => {
                if obj.is_bool() {
                    Self::Bool
                } else {
                    unimplemented!();
                }
            }
            Ty::Float => Self::Float,
            Ty::Ptr => {
                let obj_type = obj.obj_type().unwrap();
                if obj_type == Str::OBJ_TYPE {
                    Self::Str
                } else {
                    Self::Obj(obj_type)
                }
            }
        }
    }
}

impl<'a> ArgValue<'a> {
    fn from_obj(obj: &'a Obj) -> Self {
        match ArgType::of(obj) {
            ArgType::Int => Self::Int(obj.to_int()),
            ArgType::Str => Self::Str(obj.get_str().unwrap()),
            ArgType::None => Self::None,
            ArgType::Bool => Self::Bool(obj.try_to_bool().unwrap()),
            ArgType::Float => Self::Float(obj.to_float()),
            ArgType::Obj(_) => Self::Obj(*obj),
        }
    }

    fn ty(self) -> ArgType {
        match self {
            Self::Int(_) => ArgType::Int,
            Self::Str(_) => ArgType::Str,
            Self::None => ArgType::None,
            Self::Bool(_) => ArgType::Bool,
            Self::Float(_) => ArgType::Float,
            Self::Obj(o) => ArgType::Obj(o.obj_type().unwrap()),
        }
    }

    pub fn as_int(self) -> i32 {
        match self {
            Self::Int(int) => int,
            _ => panic!(),
        }
    }

    pub fn as_str(self) -> &'a [u8] {
        match self {
            Self::Str(str) => str,
            _ => panic!(),
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            _ => panic!(),
        }
    }

    pub fn as_float(self) -> f32 {
        match self {
            Self::Float(float) => float,
            _ => panic!(),
        }
    }

    pub fn as_obj(self) -> Obj {
        match self {
            Self::Obj(obj) => obj,
            _ => panic!(),
        }
    }
}

impl<'a> Arg<'a> {
    pub const fn value(&self) -> ArgValue<'a> {
        match self {
            Self::Positional(value) => *value,
            Self::Keyword(kw_arg) => kw_arg.value,
        }
    }
}

impl<'a> Args<'a> {
    pub const unsafe fn from_ptr(n_pos: usize, n_kw: usize, ptr: *const Obj) -> Self {
        let len = n_pos + (n_kw * 2);
        Self {
            n_pos,
            n_kw,
            args: unsafe { std::slice::from_raw_parts(ptr, len) },
        }
    }

    pub const fn from_fun1_args(args: &'a [Obj]) -> Self {
        Self {
            n_pos: 1,
            n_kw: 0,
            args,
        }
    }

    pub const fn from_fun2_args(args: &'a [Obj]) -> Self {
        Self {
            n_pos: 2,
            n_kw: 0,
            args,
        }
    }

    pub const fn from_fun3_args(args: &'a [Obj]) -> Self {
        Self {
            n_pos: 3,
            n_kw: 0,
            args,
        }
    }

    pub fn nth(&self, n: usize) -> Result<Arg<'a>, ArgError> {
        if n < self.n_pos {
            return Ok(Arg::Positional(ArgValue::from_obj(&self.args[n])));
        }

        let kw_index = n - self.n_pos;
        if kw_index > self.n_kw {
            return Err(ArgError::NotPresent);
        }

        let array_index = (kw_index * 2) + self.n_pos;
        Ok(Arg::Keyword(KwArg {
            kw: self.args[array_index].get_str().unwrap(),
            value: ArgValue::from_obj(&self.args[array_index + 1]),
        }))
    }

    pub fn nth_with_type(&self, n: usize, ty: ArgType) -> Result<Arg<'a>, ArgError> {
        let arg = self.nth(n)?;
        let arg_ty = arg.value().ty();
        if ty == arg_ty {
            Ok(arg)
        } else {
            Err(ArgError::TypeMismatch {
                n,
                expected: ty,
                found: arg_ty,
            })
        }
    }

    pub const fn reader(self, token: InitToken) -> ArgsReader<'a> {
        ArgsReader {
            args: self,
            n: 0,
            token,
        }
    }
}

impl<'a> ArgsReader<'a> {
    pub fn assert_npos(&self, min: usize, max: usize) -> &Self {
        if self.args.n_pos < min || self.args.n_pos > max {
            if max == 0 {
                raise_type_error(
                    self.token,
                    format!("function does not accept positional arguments"),
                )
            } else {
                raise_type_error(
                    self.token,
                    format!(
                        "function expects at least {min} positional arguments and at most {max}"
                    ),
                )
            }
        }
        self
    }

    pub fn assert_nkw(&self, min: usize, max: usize) -> &Self {
        if self.args.n_kw < min || self.args.n_kw > max {
            if max == 0 {
                raise_type_error(
                    self.token,
                    format!("function does not accept keyword arguments"),
                )
            } else {
                raise_type_error(
                    self.token,
                    format!("function expects at least {min} keyword arguments and at most {max}"),
                )
            }
        }
        self
    }

    pub fn try_next_positional(&mut self, ty: ArgType) -> Result<ArgValue<'a>, ArgError> {
        if self.n < self.args.n_pos {
            let arg = self.args.nth_with_type(self.n, ty).map(|arg| arg.value())?;
            self.n += 1;
            Ok(arg)
        } else {
            Err(ArgError::PositionalsExhuasted { n: self.n })
        }
    }

    pub fn try_next_positional_or(
        &mut self,
        ty: ArgType,
        default: ArgValue<'a>,
    ) -> Result<ArgValue<'a>, ArgError> {
        match self.try_next_positional(ty) {
            Ok(v) => Ok(v),
            Err(e) => match e {
                ArgError::PositionalsExhuasted { .. } => Ok(default),
                _ => Err(e),
            },
        }
    }

    pub fn next_positional(&mut self, ty: ArgType) -> ArgValue<'a> {
        self.try_next_positional(ty)
            .unwrap_or_else(|e| e.raise_positional(self.token))
    }

    pub fn next_positional_or(&mut self, ty: ArgType, default: ArgValue<'a>) -> ArgValue<'a> {
        self.try_next_positional_or(ty, default)
            .unwrap_or_else(|e| e.raise_positional(self.token))
    }

    pub fn try_get_kw(&self, kw: &[u8], ty: ArgType) -> Result<ArgValue<'a>, ArgError> {
        for i in 0..self.args.n_kw {
            let arg = self.args.nth(self.args.n_pos + i).unwrap();
            match arg {
                Arg::Keyword(KwArg { kw: arg_kw, value }) => {
                    if kw == arg_kw && ty == value.ty() {
                        return Ok(value);
                    }
                }
                Arg::Positional(_) => unreachable!(),
            }
        }
        Err(ArgError::NotPresent)
    }

    pub fn try_get_kw_or(
        &self,
        kw: &[u8],
        ty: ArgType,
        default: ArgValue<'a>,
    ) -> Result<ArgValue<'a>, ArgError> {
        match self.try_get_kw(kw, ty) {
            Ok(arg) => Ok(arg),
            Err(err) => match err {
                ArgError::NotPresent { .. } => Ok(default),
                _ => Err(err),
            },
        }
    }

    pub fn get_kw(&self, kw: &[u8], ty: ArgType) -> ArgValue<'a> {
        self.try_get_kw(kw, ty)
            .unwrap_or_else(|e| e.raise_kw(self.token, str::from_utf8(kw).unwrap()))
    }

    pub fn get_kw_or(&self, kw: &[u8], ty: ArgType, default: ArgValue<'a>) -> ArgValue<'a> {
        self.try_get_kw_or(kw, ty, default)
            .unwrap_or_else(|e| e.raise_kw(self.token, str::from_utf8(kw).unwrap()))
    }

    pub fn try_next_kw(&mut self, ty: ArgType) -> Result<KwArg<'a>, ArgError> {
        match self.args.nth_with_type(self.n, ty) {
            Ok(arg) => match arg {
                Arg::Keyword(kw_arg) => Ok(kw_arg),
                Arg::Positional(_) => Err(ArgError::ExpectedKeyword),
            },
            Err(e) => Err(match e {
                ArgError::NotPresent => ArgError::KeywordsExhuasted,
                ArgError::TypeMismatch { .. } => e,
                _ => unreachable!(),
            }),
        }
    }

    pub fn next_kw(&mut self, ty: ArgType) -> KwArg<'a> {
        self.try_next_kw(ty)
            .unwrap_or_else(|e| e.raise_kw(self.token, ""))
    }
}

impl Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Str => write!(f, "str"),
            Self::None => write!(f, "None"),
            Self::Bool => write!(f, "bool"),
            Self::Float => write!(f, "float"),
            Self::Obj(ty) => write!(
                f,
                "{}",
                str::from_utf8(ty.name().bytes()).expect("invalid utf8 type name wtf")
            ),
        }
    }
}

impl ArgError {
    pub fn raise_positional(&self, token: InitToken) -> ! {
        match self {
            Self::PositionalsExhuasted { n } => raise_type_error(
                token,
                // TODO: this may be confusing when a function accepts more than n + 1 arguments
                format!("expected at least {} positional arguments", n + 1),
            ),
            Self::TypeMismatch { n, expected, found } => raise_type_error(
                token,
                format!(
                    "expected type <{expected}> for argument #{}, found type <{found}>",
                    n + 1
                ),
            ),
            _ => panic!("invalid positional arg error"),
        }
    }

    pub fn raise_kw(&self, token: InitToken, arg_name: impl AsRef<str>) -> ! {
        match self {
            Self::TypeMismatch { n, expected, found } => raise_type_error(
                token,
                format!(
                    "expected type <{expected}> for argument #{}, found <{found}>",
                    n + 1
                ),
            ),
            Self::NotPresent => raise_type_error(
                token,
                format!("expected keyword argument '{}'", arg_name.as_ref()),
            ),
            Self::ExpectedKeyword => raise_type_error(
                token,
                format!("expected keyword argument instead of positional"),
            ),
            Self::KeywordsExhuasted => {
                raise_type_error(token, format!("expected keyword argument"))
            }
            _ => panic!("invalid kw arg error"),
        }
    }
}
