use std::fmt::Display;

use micropython_rs::{
    except::raise_type_error,
    init::InitToken,
    obj::{Obj, ObjTrait, ObjType, repr_c},
    str::Str,
};

use crate::error_msg::error_msg;

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

pub trait ArgTrait<'a>: Sized {
    fn ty() -> ArgType<'a>;
    fn from_arg_value(v: ArgValue<'a>) -> Option<Self>;

    fn coercable(ty: ArgType<'a>) -> bool {
        Self::ty() == ty
    }
}

impl<'a> ArgTrait<'a> for i32 {
    fn ty() -> ArgType<'a> {
        ArgType::Int
    }

    fn from_arg_value(v: ArgValue<'a>) -> Option<Self> {
        match v {
            ArgValue::Int(i) => Some(i),
            _ => None,
        }
    }
}

impl<'a> ArgTrait<'a> for &'a str {
    fn ty() -> ArgType<'static> {
        ArgType::Str
    }

    fn from_arg_value(v: ArgValue<'a>) -> Option<Self> {
        match v {
            ArgValue::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> ArgTrait<'a> for bool {
    fn ty() -> ArgType<'static> {
        ArgType::Bool
    }

    fn from_arg_value(v: ArgValue<'a>) -> Option<Self> {
        match v {
            ArgValue::Bool(b) => Some(b),
            _ => None,
        }
    }
}

impl<'a> ArgTrait<'a> for f32 {
    fn ty() -> ArgType<'static> {
        ArgType::Float
    }

    fn from_arg_value(v: ArgValue<'a>) -> Option<Self> {
        match v {
            ArgValue::Float(f) => Some(f),
            ArgValue::Int(i) => Some(i as f32),
            _ => None,
        }
    }

    fn coercable(ty: ArgType<'a>) -> bool {
        match ty {
            ArgType::Float | ArgType::Int => true,
            _ => false,
        }
    }
}

impl<'a, O: ObjTrait> ArgTrait<'a> for &'a O {
    fn ty() -> ArgType<'static> {
        ArgType::Obj(O::OBJ_TYPE)
    }

    fn from_arg_value(v: ArgValue<'a>) -> Option<Self> {
        match v {
            ArgValue::Obj(o) => o.try_as_obj_or_coerce(),
            _ => None,
        }
    }

    fn coercable(ty: ArgType<'a>) -> bool {
        match ty {
            ArgType::Obj(o) => O::OBJ_TYPE == o || O::coercable(o),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgType<'a> {
    Int,
    Str,
    None,
    Bool,
    Float,
    Obj(&'a ObjType),
}

#[derive(Clone, Copy)]
pub enum ArgValue<'a> {
    Int(i32),
    Str(&'a str),
    None,
    Bool(bool),
    Float(f32),
    Obj(&'a Obj),
}

#[derive(Clone, Copy)]
pub struct KwArg<'a> {
    pub kw: &'a str,
    pub value: ArgValue<'a>,
}

#[derive(Clone, Copy)]
pub struct GenericKwArg<'a, A: ArgTrait<'a>> {
    pub kw: &'a str,
    pub value: A,
}

#[derive(Clone, Copy)]
pub enum Arg<'a> {
    Positional(ArgValue<'a>),
    Keyword(KwArg<'a>),
}

#[derive(Debug)]
pub enum ArgError<'a> {
    NotPresent,
    TypeMismatch {
        n: usize,
        expected: ArgType<'a>,
        found: ArgType<'a>,
    },
    PositionalsExhuasted {
        n: usize,
    },
    ExpectedKeyword,
    KeywordsExhuasted,
}

impl<'a> ArgType<'a> {
    pub fn of(obj: &'a Obj) -> Self {
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
    pub fn from_obj(obj: &'a Obj) -> Self {
        match ArgType::of(obj) {
            ArgType::Int => Self::Int(obj.to_int()),
            ArgType::Str => Self::Str(obj.get_str().unwrap()),
            ArgType::None => Self::None,
            ArgType::Bool => Self::Bool(obj.try_to_bool().unwrap()),
            ArgType::Float => Self::Float(obj.to_float()),
            ArgType::Obj(_) => Self::Obj(obj),
        }
    }

    pub fn ty(self) -> ArgType<'a> {
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

    pub fn as_str(self) -> &'a str {
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
            Self::Obj(obj) => *obj,
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
    pub const fn new(n_pos: usize, n_kw: usize, args: &'a [Obj]) -> Self {
        Self { n_pos, n_kw, args }
    }

    pub fn nth(&self, n: usize) -> Result<Arg<'a>, ArgError<'a>> {
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

    pub fn nth_of_type(&self, n: usize, ty: ArgType<'a>) -> Result<Arg<'a>, ArgError<'a>> {
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
                raise_type_error(self.token, c"function does not accept positional arguments")
            } else {
                raise_type_error(
                    self.token,
                    error_msg!(
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
                raise_type_error(self.token, c"function does not accept keyword arguments")
            } else {
                raise_type_error(
                    self.token,
                    error_msg!(
                        "function expects at least {min} keyword arguments and at most {max}"
                    ),
                )
            }
        }
        self
    }

    pub fn try_next_positional_untyped(&mut self) -> Result<ArgValue<'a>, ArgError<'a>> {
        if self.n < self.args.n_pos {
            let arg = self.args.nth(self.n).map(|arg| arg.value())?;
            self.n += 1;
            Ok(arg)
        } else {
            Err(ArgError::PositionalsExhuasted { n: self.n })
        }
    }

    pub fn try_next_positional<A: ArgTrait<'a>>(&mut self) -> Result<A, ArgError<'a>> {
        if self.n < self.args.n_pos {
            let arg = self
                .args
                .nth_of_type(self.n, A::ty())
                .map(|arg| arg.value())?;
            self.n += 1;
            Ok(unsafe { ArgTrait::from_arg_value(arg).unwrap_unchecked() })
        } else {
            Err(ArgError::PositionalsExhuasted { n: self.n })
        }
    }

    pub fn try_next_positional_or<A: ArgTrait<'a>>(
        &mut self,
        default: A,
    ) -> Result<A, ArgError<'a>> {
        match self.try_next_positional() {
            Ok(v) => Ok(v),
            Err(e) => match e {
                ArgError::PositionalsExhuasted { .. } => Ok(default),
                _ => Err(e),
            },
        }
    }

    pub fn next_positional<A: ArgTrait<'a>>(&mut self) -> A {
        // borrow checker moment
        let token = self.token;
        self.try_next_positional()
            .unwrap_or_else(|e| e.raise_positional(token))
    }

    pub fn next_positional_or<A: ArgTrait<'a>>(&mut self, default: A) -> A {
        let token = self.token;
        self.try_next_positional_or(default)
            .unwrap_or_else(|e| e.raise_positional(token))
    }

    pub fn try_get_kw<A: ArgTrait<'a>>(&self, kw: &str) -> Result<A, ArgError<'a>> {
        for i in 0..self.args.n_kw {
            let arg = self.args.nth(self.args.n_pos + i).unwrap();
            match arg {
                Arg::Keyword(KwArg { kw: arg_kw, value }) => {
                    if kw == arg_kw && A::ty() == value.ty() {
                        return Ok(unsafe { A::from_arg_value(value).unwrap_unchecked() });
                    }
                }
                Arg::Positional(_) => unreachable!(),
            }
        }
        Err(ArgError::NotPresent)
    }

    pub fn try_get_kw_or<A: ArgTrait<'a>>(&self, kw: &str, default: A) -> Result<A, ArgError<'a>> {
        match self.try_get_kw(kw) {
            Ok(arg) => Ok(arg),
            Err(err) => match err {
                ArgError::NotPresent { .. } => Ok(default),
                _ => Err(err),
            },
        }
    }

    pub fn get_kw<A: ArgTrait<'a>>(&self, kw: &str) -> A {
        self.try_get_kw(kw)
            .unwrap_or_else(|e| e.raise_kw(self.token, kw))
    }

    pub fn get_kw_or<A: ArgTrait<'a>>(&self, kw: &str, default: A) -> A {
        self.try_get_kw_or(kw, default)
            .unwrap_or_else(|e| e.raise_kw(self.token, kw))
    }

    pub fn try_next_kw_untyped(&mut self) -> Result<KwArg<'a>, ArgError<'a>> {
        match self.args.nth(self.n) {
            Ok(arg) => match arg {
                Arg::Keyword(kw_arg) => {
                    self.n += 1;
                    Ok(kw_arg)
                }
                Arg::Positional(_) => Err(ArgError::ExpectedKeyword),
            },
            Err(e) => Err(match e {
                ArgError::NotPresent => ArgError::KeywordsExhuasted,
                _ => unreachable!(),
            }),
        }
    }

    pub fn try_next_kw<A: ArgTrait<'a>>(&mut self) -> Result<GenericKwArg<'a, A>, ArgError<'a>> {
        match self.args.nth_of_type(self.n, A::ty()) {
            Ok(arg) => match arg {
                Arg::Keyword(kw_arg) => {
                    self.n += 1;
                    Ok(GenericKwArg {
                        kw: kw_arg.kw,
                        value: unsafe { A::from_arg_value(kw_arg.value).unwrap_unchecked() },
                    })
                }
                Arg::Positional(_) => Err(ArgError::ExpectedKeyword),
            },
            Err(e) => Err(match e {
                ArgError::NotPresent => ArgError::KeywordsExhuasted,
                ArgError::TypeMismatch { .. } => e,
                _ => unreachable!(),
            }),
        }
    }

    pub fn next_kw<A: ArgTrait<'a>>(&mut self) -> GenericKwArg<'a, A> {
        let token = self.token;
        self.try_next_kw().unwrap_or_else(|e| e.raise_kw(token, ""))
    }
}

impl Display for ArgType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Str => write!(f, "str"),
            Self::None => write!(f, "None"),
            Self::Bool => write!(f, "bool"),
            Self::Float => write!(f, "float"),
            Self::Obj(ty) => write!(f, "{}", ty.name().as_str()),
        }
    }
}

impl ArgError<'_> {
    pub fn raise_positional(&self, token: InitToken) -> ! {
        match self {
            Self::PositionalsExhuasted { n } => raise_type_error(
                token,
                // TODO: this may be confusing when a function accepts more than n + 1 arguments
                error_msg!("expected at least {} positional arguments", n + 1),
            ),
            Self::TypeMismatch { n, expected, found } => raise_type_error(
                token,
                error_msg!(
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
                error_msg!(
                    "expected type <{expected}> for argument #{}, found <{found}>",
                    n + 1
                ),
            ),
            Self::NotPresent => raise_type_error(
                token,
                error_msg!("expected keyword argument '{}'", arg_name.as_ref()),
            ),
            Self::ExpectedKeyword => {
                raise_type_error(token, c"expected keyword argument instead of positional")
            }
            Self::KeywordsExhuasted => raise_type_error(token, c"expected keyword argument"),
            _ => panic!("invalid kw arg error"),
        }
    }
}
