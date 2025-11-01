use std::fmt::Display;

use micropython_rs::{
    except::raise_type_error,
    init::InitToken,
    obj::{Obj, ObjTrait, ObjType},
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
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgType {
    None,
    Int,
    Bool,
    Str,
    Obj(&'static ObjType),
}

#[derive(Clone, Copy)]
pub enum ArgValue<'a> {
    None,
    Int(i32),
    Bool(bool),
    Str(&'a [u8]),
    Obj(Obj),
}

#[derive(Clone, Copy)]
pub enum Arg<'a> {
    Positional(ArgValue<'a>),
    Keyword { kw: &'a [u8], value: ArgValue<'a> },
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
}

impl ArgType {
    fn of(obj: &Obj) -> Self {
        let int = obj.as_ptr() as u32;
        // - `xxxx...xxx1` is a small int, and bits 1 and above are the value
        // - `xxxx...x010` is a qstr, and bits 3 and above are the value
        // - `xxxx...x110` is an immediate object, and bits 3 and abvoe are the value
        // - `xxxx...xx00` is a pointer to an [`ObjBase`]
        match int & 0b111 {
            0b010 => Self::Str,
            0b110 => match obj.as_immediate().unwrap() {
                0 => Self::None,
                3 | 1 => Self::Bool,
                _ => unimplemented!(),
            },
            _ => match int & 0b11 {
                0b00 => {
                    if obj.is(Str::OBJ_TYPE) {
                        Self::Str
                    } else {
                        Self::Obj(obj.obj_type().unwrap())
                    }
                }
                _ => match int & 0b1 {
                    0b1 => Self::Int,
                    _ => unreachable!(),
                },
            },
        }
    }
}

impl<'a> ArgValue<'a> {
    fn from_obj(obj: &'a Obj) -> Self {
        match ArgType::of(obj) {
            ArgType::None => Self::None,
            ArgType::Int => Self::Int(obj.as_small_int().unwrap()),
            ArgType::Str => Self::Str(obj.get_str().unwrap()),
            ArgType::Bool => Self::Bool(obj.as_bool().unwrap()),
            ArgType::Obj(_) => Self::Obj(*obj),
        }
    }

    fn ty(self) -> ArgType {
        match self {
            Self::None => ArgType::None,
            Self::Int(_) => ArgType::Int,
            Self::Str(_) => ArgType::Str,
            Self::Bool(_) => ArgType::Bool,
            Self::Obj(o) => ArgType::Obj(o.obj_type().unwrap()),
        }
    }

    pub fn as_int(self) -> Option<i32> {
        match self {
            Self::Int(int) => Some(int),
            _ => None,
        }
    }

    pub fn as_str(self) -> Option<&'a [u8]> {
        match self {
            Self::Str(str) => Some(str),
            _ => None,
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        match self {
            Self::Bool(bool) => Some(bool),
            _ => None,
        }
    }

    pub fn as_obj(self) -> Option<Obj> {
        match self {
            Self::Obj(obj) => Some(obj),
            _ => None,
        }
    }
}

impl<'a> Arg<'a> {
    pub const fn value(&self) -> ArgValue<'a> {
        match self {
            Self::Positional(value) => *value,
            Self::Keyword { value, .. } => *value,
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
        Ok(Arg::Keyword {
            kw: self.args[array_index].get_str().unwrap(),
            value: ArgValue::from_obj(&self.args[array_index]),
        })
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

    pub const fn reader(self) -> ArgsReader<'a> {
        ArgsReader { args: self, n: 0 }
    }
}

impl<'a> ArgsReader<'a> {
    pub fn next_positional(&mut self, ty: ArgType) -> Result<ArgValue<'a>, ArgError> {
        if self.n < self.args.n_pos {
            let arg = self.args.nth_with_type(self.n, ty).map(|arg| arg.value());
            self.n += 1;
            arg
        } else {
            Err(ArgError::PositionalsExhuasted { n: self.n })
        }
    }

    pub fn get_kw(&self, kw: &[u8], ty: ArgType) -> Result<ArgValue<'a>, ArgError> {
        for i in 0..self.args.n_kw {
            let arg = self.args.nth(self.args.n_pos + i).unwrap();
            match arg {
                Arg::Keyword { kw: arg_kw, value } => {
                    if kw == arg_kw && ty == value.ty() {
                        return Ok(value);
                    }
                }
                Arg::Positional(_) => unreachable!(),
            }
        }
        Err(ArgError::NotPresent)
    }

    pub fn next_positional_or<'b: 'a>(
        &mut self,
        ty: ArgType,
        default: ArgValue<'b>,
    ) -> Result<ArgValue<'a>, ArgError> {
        match self.next_positional(ty) {
            Ok(arg) => Ok(arg),
            Err(err) => match err {
                ArgError::PositionalsExhuasted { .. } => Ok(default),
                _ => Err(err),
            },
        }
    }

    pub fn get_kw_or<'b: 'a>(
        &self,
        kw: &[u8],
        ty: ArgType,
        default: ArgValue<'b>,
    ) -> Result<ArgValue<'a>, ArgError> {
        match self.get_kw(kw, ty) {
            Ok(arg) => Ok(arg),
            Err(err) => match err {
                ArgError::NotPresent { .. } => Ok(default),
                _ => Err(err),
            },
        }
    }
}

impl Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::None => write!(f, "None"),
            Self::Str => write!(f, "str"),
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
                    "expected type '{expected}' for argument {}, found '{found}'",
                    n + 1
                ),
            ),
            Self::NotPresent => raise_type_error(
                token,
                format!("expected keyword argument '{}'", arg_name.as_ref()),
            ),
            _ => panic!("invalid kw arg error"),
        }
    }
}
