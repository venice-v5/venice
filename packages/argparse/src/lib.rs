mod error_msg;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::RangeInclusive,
};

use micropython_rs::{
    except::{Exception, Message, type_error, value_error},
    init::token,
    obj::{Obj, ObjTrait, ObjType, repr_c},
    str::Str,
};

pub use crate::error_msg::*;

#[derive(Clone, Copy)]
pub struct Args<'a> {
    pos_args: &'a [Obj],
    kw_args: &'a [Obj],
}

#[derive(Clone, Copy)]
pub struct ArgsReader<'a> {
    args: Args<'a>,
    n_pos: usize,
    n_kw: usize,
}

pub trait ArgParser<'a> {
    type Output;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError>;
}

pub trait DefaultParser<'a> {
    type Parser: ArgParser<'a, Output = Self> + Default;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StrParser;

#[derive(Debug, Clone)]
pub struct IntParser<T = i32> {
    pub range: RangeInclusive<i32>,
    pub _phantom: PhantomData<T>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FloatParser;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoolParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjParser<T: ObjTrait> {
    pub _phantom: PhantomData<T>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AnyParser;

impl<'a> ArgParser<'a> for StrParser {
    type Output = &'a str;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        obj.get_str()
            .ok_or(ParseError::TypeError { expected: "str" })
    }
}

impl<'a> DefaultParser<'a> for &'a str {
    type Parser = StrParser;
}

impl<T> IntParser<T> {
    pub const fn new(range: RangeInclusive<i32>) -> Self {
        Self {
            range,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> ArgParser<'a> for IntParser<T>
where
    T: TryFrom<i32>,
    <T as TryFrom<i32>>::Error: Debug,
{
    type Output = T;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        let int = obj
            .try_to_int()
            .ok_or(ParseError::TypeError { expected: "int" })?;

        if !self.range.contains(&int) {
            let start = *self.range.start();
            let end = *self.range.end();

            return Err(ParseError::ValueError {
                mk_msg: Box::new(move |arg| {
                    error_msg!("{arg} must be in the range [{start}, {end}]")
                }),
            });
        }

        Ok(int.try_into().expect("value "))
    }
}

macro_rules! impl_default_int_parser {
    ($ty:ty) => {
        impl Default for IntParser<$ty> {
            fn default() -> Self {
                Self::new((<$ty>::MIN as i32)..=(<$ty>::MAX as i32))
            }
        }

        impl DefaultParser<'_> for $ty {
            type Parser = IntParser<$ty>;
        }
    };
}

impl_default_int_parser!(u8);
impl_default_int_parser!(u16);
impl_default_int_parser!(u32);

impl_default_int_parser!(i8);
impl_default_int_parser!(i16);
impl_default_int_parser!(i32);

impl<'a> ArgParser<'a> for FloatParser {
    type Output = f32;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        obj.try_to_float()
            .or_else(|| obj.try_to_int().map(|int| int as f32))
            .ok_or(ParseError::TypeError { expected: "float" })
    }
}

impl DefaultParser<'_> for f32 {
    type Parser = FloatParser;
}

impl<'a> ArgParser<'a> for BoolParser {
    type Output = bool;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        obj.try_to_bool()
            .ok_or(ParseError::TypeError { expected: "bool" })
    }
}

impl DefaultParser<'_> for bool {
    type Parser = BoolParser;
}

impl<'a, T> ArgParser<'a> for ObjParser<T>
where
    T: ObjTrait + 'a,
{
    type Output = &'a T;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        obj.try_as_obj::<T>().ok_or(ParseError::TypeError {
            expected: T::OBJ_TYPE.name().as_str(),
        })
    }
}

impl<T: ObjTrait> Default for ObjParser<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> DefaultParser<'a> for &'a T
where
    T: ObjTrait + 'a,
{
    type Parser = ObjParser<T>;
}

impl<'a> ArgParser<'a> for AnyParser {
    type Output = Obj;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        Ok(*obj)
    }
}

impl DefaultParser<'_> for Obj {
    type Parser = AnyParser;
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
pub enum Arg<'a> {
    Positional(&'a Obj),
    Keyword(&'a str, &'a Obj),
}

#[derive(Clone, Copy)]
pub struct KeywordArg<'a> {
    pub kw: &'a str,
    pub obj: &'a Obj,
}

pub enum ParseError {
    TypeError {
        expected: &'static str,
    },
    ValueError {
        mk_msg: Box<dyn FnOnce(&str) -> Message>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum PositionalError<'a> {
    ArgumentsExhausted,
    TypeError {
        n: usize,
        expected: &'a str,
        found: &'a str,
    },
    ValueError {
        msg: Message,
    },
}

pub enum KeywordError<'a> {
    TypeError {
        kw: &'a str,
        expected: &'a str,
        found: &'a str,
    },
    ValueError {
        msg: Message,
    },
}

impl From<PositionalError<'_>> for Exception {
    fn from(value: PositionalError<'_>) -> Self {
        match value {
            PositionalError::ArgumentsExhausted => {
                type_error(c"unexpected end of positional arguments")
            }
            PositionalError::TypeError { n, expected, found } => type_error(error_msg!(
                "expected '{expected}' for argument #{n}, found '{found}'"
            )),
            PositionalError::ValueError { msg } => value_error(msg),
        }
    }
}

impl From<KeywordError<'_>> for Exception {
    fn from(value: KeywordError<'_>) -> Self {
        match value {
            KeywordError::TypeError {
                kw,
                expected,
                found,
            } => type_error(error_msg!(
                "expected '{expected}' found argument '{kw}', found '{found}'"
            )),
            KeywordError::ValueError { msg } => value_error(msg),
        }
    }
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

pub fn type_name(obj: &Obj) -> &'static str {
    match ArgType::of(obj) {
        ArgType::Int => "int",
        ArgType::Bool => "bool",
        ArgType::Str => "str",
        ArgType::None => "None",
        ArgType::Float => "float",
        ArgType::Obj(o) => o.name().as_str(),
    }
}

impl<'a> KeywordArg<'a> {
    pub fn parse<T>(&self) -> Result<T, KeywordError<'a>>
    where
        T: DefaultParser<'a>,
    {
        let parser = T::Parser::default();
        match parser.parse(self.obj) {
            Ok(v) => Ok(v),
            Err(e) => Err(match e {
                ParseError::TypeError { expected } => KeywordError::TypeError {
                    kw: self.kw,
                    expected,
                    found: type_name(self.obj),
                },
                ParseError::ValueError { mk_msg } => KeywordError::ValueError {
                    msg: mk_msg(&format!("argument '{}'", self.kw)),
                },
            }),
        }
    }
}

impl<'a> Args<'a> {
    pub fn new(n_pos: usize, n_kw: usize, args: &'a [Obj]) -> Self {
        Self {
            pos_args: &args[..n_pos],
            kw_args: &args[n_pos..n_pos + (n_kw * 2)],
        }
    }

    pub fn count(&self) -> usize {
        self.pos_args.len() + self.kw_args.len() / 2
    }

    pub fn nth_pos(&self, n: usize) -> Option<&'a Obj> {
        self.pos_args.get(n)
    }

    pub fn nth_kw(&self, n: usize) -> Option<KeywordArg<'a>> {
        let index = n * 2;
        let kw = self.kw_args.get(index)?.get_str().unwrap();
        let obj = self.kw_args.get(index + 1)?;

        Some(KeywordArg { kw, obj })
    }

    pub const fn reader(self) -> ArgsReader<'a> {
        ArgsReader {
            args: self,
            n_pos: 0,
            n_kw: 0,
        }
    }
}

impl<'a> ArgsReader<'a> {
    pub fn assert_npos(&self, min: usize, max: usize) -> &Self {
        if !(min..=max).contains(&self.args.pos_args.len()) {
            if max == 0 {
                type_error(c"function does not accept positional arguments").raise(token())
            } else {
                type_error(error_msg!(
                    "function expects at least {min} positional arguments and at most {max}"
                ))
                .raise(token())
            }
        }
        self
    }

    pub fn assert_nkw(&self, min: usize, max: usize) -> &Self {
        if !(min..=max).contains(&(self.args.kw_args.len() / 2)) {
            if max == 0 {
                type_error(c"function does not accept keyword arguments").raise(token())
            } else {
                type_error(error_msg!(
                    "function expects at least {min} keyword arguments and at most {max}"
                ))
                .raise(token())
            }
        }
        self
    }

    pub fn next_positional_with<P>(&mut self, parser: P) -> Result<P::Output, PositionalError<'a>>
    where
        P: ArgParser<'a>,
    {
        self.args
            .nth_pos(self.n_pos)
            .ok_or(PositionalError::ArgumentsExhausted)
            .and_then(|arg| {
                let result = parser.parse(arg);
                result
                    .map_err(|e| match e {
                        ParseError::TypeError { expected } => {
                            let found = type_name(arg);
                            PositionalError::TypeError {
                                n: self.n_pos,
                                expected,
                                found,
                            }
                        }
                        ParseError::ValueError { mk_msg } => PositionalError::ValueError {
                            msg: mk_msg(&format!("argument #{}", self.n_pos)),
                        },
                    })
                    .inspect(|_| self.n_pos += 1)
            })
    }

    pub fn next_positional<T>(&mut self) -> Result<T, PositionalError<'a>>
    where
        T: DefaultParser<'a>,
    {
        self.next_positional_with(T::Parser::default())
    }

    pub fn next_positional_or<T>(&mut self, default: T) -> Result<T, PositionalError<'a>>
    where
        T: DefaultParser<'a>,
    {
        match self.next_positional_with(T::Parser::default()) {
            Ok(v) => Ok(v),
            Err(e) => match e {
                PositionalError::ArgumentsExhausted => Ok(default),
                _ => Err(e),
            },
        }
    }

    pub fn next_kw(&mut self) -> Option<KeywordArg<'a>> {
        self.args.nth_kw(self.n_kw).inspect(|_| self.n_kw += 1)
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
