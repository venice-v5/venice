use std::{
    ffi::{CStr, c_void},
    fmt::Debug,
    marker::PhantomData,
    ops::RangeInclusive,
    ptr::NonNull,
};

use bytemuck::{AnyBitPattern, PodCastError};
use micropython_rs::{
    buffer::{Buffer, BufferError},
    obj::{Obj, ObjTrait, ObjType},
};

use crate::{ArgParser, DefaultParser, ParseError, error_msg};

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

#[derive(Debug)]
pub struct BufferParser<T>
where
    T: AnyBitPattern,
{
    pub _phantom: PhantomData<T>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CallableParser;

#[derive(Clone, Copy)]
pub struct Callable(NonNull<c_void>);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CStrParser;

#[derive(Clone, Copy)]
pub struct RawObjParser {
    pub ty: &'static ObjType,
}

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

impl<'a, T> ArgParser<'a> for BufferParser<T>
where
    T: AnyBitPattern,
{
    type Output = Buffer<'a, T>;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        let buffer = obj.buffer().map_err(|e| match e {
            BufferError::NonBuffer => ParseError::TypeError {
                expected: "buffer object (e.g. 'array' or 'memoryview')",
            },
            BufferError::BufferUnavailable => ParseError::ValueError {
                mk_msg: Box::new(|arg| error_msg!("{arg} is unavailable for reading")),
            },
        })?;

        buffer.cast().map_err(|e| match e {
            PodCastError::OutputSliceWouldHaveSlop => ParseError::ValueError {
                mk_msg: Box::new(|arg| {
                    error_msg!("{arg} length must be a multiple of {}", size_of::<T>())
                }),
            },
            PodCastError::TargetAlignmentGreaterAndInputNotAligned => {
                panic!("buffer unaligned")
            }
            _ => unreachable!(),
        })
    }
}

impl<T> Default for BufferParser<T>
where
    T: AnyBitPattern,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> DefaultParser<'a> for Buffer<'a, T>
where
    T: AnyBitPattern,
{
    type Parser = BufferParser<T>;
}

impl<'a> ArgParser<'a> for CallableParser {
    type Output = Callable;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        if !obj.is_callable() {
            return Err(ParseError::TypeError {
                expected: "callable object",
            });
        }

        Ok(Callable(unsafe { NonNull::new_unchecked(obj.inner()) }))
    }
}

impl DefaultParser<'_> for Callable {
    type Parser = CallableParser;
}

impl Callable {
    pub fn into_inner(self) -> Obj {
        unsafe { Obj::from_ptr(self.0.as_ptr()) }
    }

    pub fn call(&self, n_kw: usize, args: &[Obj]) -> Obj {
        unsafe { self.into_inner().call(n_kw, args).unwrap_unchecked() }
    }
}

impl<'a> ArgParser<'a> for CStrParser {
    type Output = &'a CStr;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        obj.get_cstr()
            .ok_or(ParseError::TypeError { expected: "string" })?
            .map_err(|e| ParseError::ValueError {
                mk_msg: Box::new(move |arg| {
                    error_msg!(
                        "{arg} must not contain a NUL byte, one was found at byte {}",
                        e.position()
                    )
                }),
            })
    }
}

impl<'a> DefaultParser<'a> for &'a CStr {
    type Parser = CStrParser;
}

impl<'a> ArgParser<'a> for RawObjParser {
    type Output = Obj;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, ParseError> {
        if obj.obj_type() != self.ty {
            Err(ParseError::TypeError {
                expected: self.ty.name().as_str(),
            })
        } else {
            Ok(*obj)
        }
    }
}

impl RawObjParser {
    pub fn new(ty: &'static ObjType) -> Self {
        Self { ty }
    }
}
