use std::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use bitflags::bitflags;
use thiserror::Error;

use crate::{
    gc::Gc,
    map::Dict,
    ops::{BinaryOp, UnaryOp},
    print::{Print, PrintKind},
    qstr::Qstr,
    str::Str,
};

/// From: `py/obj.h`
#[derive(Debug)]
#[repr(C)]
pub struct ObjType {
    // mp_type_type has a static lifetime
    base: ObjBase<'static>,

    flags: u16,
    name: u16,

    slot_index_make_new: u8,
    slot_index_print: u8,
    slot_index_call: u8,
    slot_index_unary_op: u8,
    slot_index_binary_op: u8,
    slot_index_attr: u8,
    slot_index_subscr: u8,
    slot_index_iter: u8,
    slot_index_buffer: u8,
    slot_index_protocol: u8,
    slot_index_parent: u8,
    slot_index_locals_dict: u8,

    slots: (),
}

/// From: `py/obj.h`
#[derive(Debug)]
#[repr(C)]
pub struct ObjFullType {
    // mp_type_type has a static lifetime
    base: ObjBase<'static>,

    flags: u16,
    name: u16,

    slot_index_make_new: u8,
    slot_index_print: u8,
    slot_index_call: u8,
    slot_index_unary_op: u8,
    slot_index_binary_op: u8,
    slot_index_attr: u8,
    slot_index_subscr: u8,
    slot_index_iter: u8,
    slot_index_buffer: u8,
    slot_index_protocol: u8,
    slot_index_parent: u8,
    slot_index_locals_dict: u8,

    slots: [*const c_void; 12],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    MakeNew = 1,
    Print = 2,
    Call = 3,
    UnaryOp = 4,
    BinaryOp = 5,
    Attr = 6,
    Subscr = 7,
    Iter = 8,
    Buffer = 9,
    Protocol = 10,
    Parent = 11,
    LocalsDict = 12,
}

/// From: `py/obj.h`
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ObjBase<'a> {
    r#type: *const ObjType,
    _phantom: PhantomData<&'a ObjType>,
}

pub type StaticObjbase = ObjBase<'static>;

/// MicroPython object
///
/// # Representation
///
/// MicroPython has four object representations:
///
/// - Repr A is optimized for 32-bit systems with 4-byte alignments. It can encode small integers,
///   interned strings (qstrs), and immediate objects (such as None, True, and False) without
///   indirection.
/// - Repr B has the same capabilities of Repr A, except that it allows for 2-byte aligned
///   pointers by increasing the amount of tagging bits used by other object types.
/// - Repr C extends Repr A with the ability to store floating-point numbers without indirection,
///   at the expense of decreasing the amount of bits allocated to qstrs and immediate objects
/// - Repr D is optimized for 64-bit systems.
///
/// ## Repr C
///
/// This port uses Repr C to optimize floating-point math, which is common in VEX programs.
///
/// - `iiiiiiii iiiiiiii iiiiiiii iiiiiii1` is a 31-bit integer
/// - `01111111 1qqqqqqq qqqqqqqq qqqq0110` is a 19-bit qstr
/// - `01111111 10000000 00000000 ssss1110` is an immediate object
/// - `seeeeeee ffffffff ffffffff ffffff10` is a 30-bit float
/// - `pppppppp pppppppp pppppppp pppppp00` is a pointer to an object, starting with [`ObjBase`]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Obj(*mut c_void);

pub mod repr_c {
    use std::ffi::c_void;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Ty {
        Int,
        Qstr,
        Immediate,
        Float,
        Ptr,
    }

    pub const fn new_int(int: i32) -> *mut c_void {
        // right shifting a signed integer (as opposed to an unsigned int) performs an arithmetic
        // right shift where the sign bit is preserved, e.g. 0b1000 >> 1 = 0b1100
        (int << 1) as *mut c_void
    }

    pub const fn new_qstr(qstr: u32) -> *mut c_void {
        (qstr << 4 | 0b110) as *mut c_void
    }

    pub const fn new_immediate(imm: u32) -> *mut c_void {
        (imm << 4 | 0b1110) as *mut c_void
    }

    pub const fn new_float(float: f32) -> *mut c_void {
        (float.to_bits().wrapping_add(0x8080_0000) & !0b11) as *mut c_void
    }

    pub const fn new_ptr(ptr: *mut c_void) -> *mut c_void {
        ptr
    }

    pub fn is_int(obj: *mut c_void) -> bool {
        (obj as u32) & 0b1 == 0b1
    }

    pub fn is_qstr(obj: *mut c_void) -> bool {
        (obj as u32) & 0xff80_000f == 0b0110
    }

    pub fn is_immediate(obj: *mut c_void) -> bool {
        (obj as u32) & 0xff80_000f == 0b1110
    }

    pub fn is_float(obj: *mut c_void) -> bool {
        let obj = obj as u32;
        (obj & 0b11 == 0b10) && (obj & 0xff80_0007 != 0b110)
    }

    pub fn is_ptr(obj: *mut c_void) -> bool {
        (obj as u32) & 0b11 == 0
    }

    pub fn get_int(obj: *mut c_void) -> i32 {
        (obj as i32) >> 1
    }

    pub fn get_qstr(obj: *mut c_void) -> u32 {
        (obj as u32) >> 4
    }

    pub fn get_immediate(obj: *mut c_void) -> u32 {
        (obj as u32) >> 4
    }

    pub fn get_float(obj: *mut c_void) -> f32 {
        f32::from_bits((obj as u32).wrapping_sub(0x8080_0000) & !0b11)
    }

    pub const fn get_ptr(obj: *mut c_void) -> *mut c_void {
        obj
    }

    pub fn type_of(obj: *mut c_void) -> Option<Ty> {
        let obj = obj as u32;
        match obj & 0b1111 {
            0b0110 => Some(match obj & 0xff80_0000 {
                0 => Ty::Qstr,
                _ => Ty::Float,
            }),
            0b1110 => Some(match obj & 0xff80_0000 {
                0 => Ty::Immediate,
                _ => Ty::Float,
            }),
            _ => match obj & 0b11 {
                0b00 => Some(Ty::Ptr),
                0b10 => Some(Ty::Float),
                0b01 | 0b11 => Some(Ty::Int),
                _ => unreachable!(),
            },
        }
    }
}

/// # Safety
///
/// Object representation must begin with an [`mp_obj_base_t`], always initialized to `OBJ_TYPE`.
/// Additionally, all instances must be aligned to exactly 4 bytes in memory, but this is already
/// guaranteed if the first invariant is true, as a side effect. A higher alignment than this will
/// cause issues, since all objects are assumed to have an inherent alignment of 4.
pub unsafe trait ObjTrait: Sized {
    const OBJ_TYPE: &ObjType;
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TypeFlags: u16 {
        const IS_SUBCLASSED = 0x0001;
        const HAS_SPECIAL_ACCESSORS = 0x0002;
        const EQ_NOT_REFLEXIVE = 0x0004;
        const EQ_CHECKS_OTHER_TYPE = 0x0008;
        const EQ_HAS_NEQ_TEST = 0x0010;
        const BINDS_SELF = 0x0020;
        const BUILTIN_FUN = 0x0040;
        const ITER_IS_GETITER = 0x0000;
        const ITER_IS_ITERNEXT = 0x0080;
        const ITER_IS_CUSTOM = 0x0100;
        const INSTANCE_TYPE = 0x0200;
    }
}

pub type MakeNewFn =
    unsafe extern "C" fn(ty: *const ObjType, n_args: usize, n_kw: usize, args: *const Obj) -> Obj;
pub type PrintFn = unsafe extern "C" fn(print: *const Print, o: Obj, kind: PrintKind);
pub type CallFn =
    unsafe extern "C" fn(fun: Obj, n_args: usize, n_kw: usize, args: *const Obj) -> Obj;
pub type UnaryOpFn = extern "C" fn(op: UnaryOp, obj: Obj) -> Obj;
pub type BinaryOpFn = extern "C" fn(op: BinaryOp, obj_1: Obj, obj_2: Obj) -> Obj;
pub type AttrFn = unsafe extern "C" fn(self_in: Obj, attr: Qstr, dest: *mut Obj);
pub type SubscrFn = extern "C" fn(self_in: Obj, index: Obj, value: Obj) -> Obj;

#[derive(Debug, Clone, Copy)]
pub struct MakeNew {
    f: MakeNewFn,
}

#[derive(Debug, Clone, Copy)]
pub struct Attr {
    f: AttrFn,
}

pub enum AttrOp<'a> {
    Load { dest: &'a mut Obj },
    Store { src: Obj },
    Delete,
}

impl MakeNew {
    pub const unsafe fn new(f: MakeNewFn) -> Self {
        Self { f }
    }
}

impl Attr {
    pub const unsafe fn new(f: AttrFn) -> Self {
        Self { f }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Subscr {
    f: SubscrFn,
}
impl Subscr {
    pub const unsafe fn new(f: SubscrFn) -> Self {
        Self { f }
    }
}
pub enum SubscrOp {
    Load,
    Store { src: Obj },
    Delete,
}

#[macro_export]
macro_rules! make_new_from_fn {
    ($f:expr) => {{
        unsafe extern "C" fn trampoline(
            ty: *const $crate::obj::ObjType,
            n_pos: usize,
            n_kw: usize,
            ptr: *const $crate::obj::Obj,
        ) -> Obj {
            // TODO: safe?
            let ty: &'static $crate::obj::ObjType = unsafe { &*ty };
            let args = unsafe { ::std::slice::from_raw_parts(ptr, n_pos + (n_kw * 2)) };
            $f(ty, n_pos, n_kw, args)
        }

        unsafe { $crate::obj::MakeNew::new(trampoline) }
    }};
}

#[macro_export]
macro_rules! attr_from_fn {
    ($f:expr) => {{
        unsafe extern "C" fn trampoline(self_in: Obj, attr: Qstr, dest: *mut Obj) {
            let op = unsafe {
                if (*dest).is_null() {
                    $crate::obj::AttrOp::Load { dest: &mut *dest }
                } else {
                    let dest_1 = dest.add(1);
                    if (*dest_1).is_null() {
                        $crate::obj::AttrOp::Delete
                    } else {
                        $crate::obj::AttrOp::Store { src: *dest_1 }
                    }
                }
            };
            $f(self_in.try_to_obj().unwrap(), attr, op)
        }

        unsafe { $crate::obj::Attr::new(trampoline) }
    }};
}

#[macro_export]
macro_rules! subscr_from_fn {
    ($f:expr) => {{
        extern "C" fn trampoline(self_in: Obj, index: Obj, value: Obj) -> Obj    {
            let Some(index) = index.try_to_int() else { return Obj::NULL };

            let op = if value.is_null() {
                SubscrOp::Delete
            } else if value.is_sentinel() {
                SubscrOp::Load
            } else {
                SubscrOp::Store { src: value }
            };

            $f(self_in.try_to_obj().unwrap(), index, op)
        }

        unsafe { $crate::obj::Subscr::new(trampoline) }
    }};
}

impl PartialEq for ObjType {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Eq for ObjType {}

impl ObjType {
    pub fn name(&self) -> Qstr {
        // SAFETY: maybe
        unsafe { Qstr::from_index(self.name as usize) }
    }
}

impl ObjFullType {
    pub const fn new(flags: TypeFlags, name: Qstr) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            flags: flags.bits(),
            name: name.index() as u16,

            slot_index_make_new: 0,
            slot_index_print: 0,
            slot_index_call: 0,
            slot_index_unary_op: 0,
            slot_index_binary_op: 0,
            slot_index_attr: 0,
            slot_index_subscr: 0,
            slot_index_iter: 0,
            slot_index_buffer: 0,
            slot_index_protocol: 0,
            slot_index_parent: 0,
            slot_index_locals_dict: 0,

            slots: [core::ptr::null(); 12],
        }
    }

    const fn slot_index(&mut self, slot: Slot) -> &mut u8 {
        match slot {
            Slot::MakeNew => &mut self.slot_index_make_new,
            Slot::Print => &mut self.slot_index_print,
            Slot::Call => &mut self.slot_index_call,
            Slot::UnaryOp => &mut self.slot_index_unary_op,
            Slot::BinaryOp => &mut self.slot_index_binary_op,
            Slot::Attr => &mut self.slot_index_attr,
            Slot::Subscr => &mut self.slot_index_subscr,
            Slot::Iter => &mut self.slot_index_iter,
            Slot::Buffer => &mut self.slot_index_buffer,
            Slot::Protocol => &mut self.slot_index_protocol,
            Slot::Parent => &mut self.slot_index_parent,
            Slot::LocalsDict => &mut self.slot_index_locals_dict,
        }
    }

    pub const fn as_obj_type(&self) -> &ObjType {
        // SAFETY: ObjType and ObjFullType have the same starting memory representation
        unsafe { std::mem::transmute(self) }
    }

    pub const unsafe fn set_slot(mut self, slot: Slot, value: *const c_void) -> Self {
        *self.slot_index(slot) = slot as u8;
        self.slots[slot as usize - 1] = value;
        self
    }

    pub const fn set_slot_locals_dict_from_static(self, value: &'static Dict) -> Self {
        unsafe { self.set_slot(Slot::LocalsDict, value as *const Dict as *const c_void) }
    }

    pub const fn set_slot_parent(self, value: &'static ObjType) -> Self {
        unsafe { self.set_slot(Slot::Parent, value as *const ObjType as *const c_void) }
    }
}

pub type IterNextFn = extern "C" fn(self_in: Obj) -> Obj;

// incomplete
pub enum IterSlotValue {
    IterNext(IterNextFn),
}

impl ObjFullType {
    // named differently because this isn't simply setting a slot
    pub const fn set_iter(mut self, iter: IterSlotValue) -> Self {
        match iter {
            IterSlotValue::IterNext(f) => {
                self.flags |= TypeFlags::ITER_IS_GETITER.bits();
                // SAFETY: f is safe to use as the iter slot value for the given iter type
                unsafe { self.set_slot_iter(f as *const c_void) }
            }
        }
    }

    pub const fn set_make_new(self, make_new: MakeNew) -> Self {
        unsafe { self.set_slot_make_new(make_new.f) }
    }

    pub const fn set_attr(self, attr: Attr) -> Self {
        unsafe { self.set_slot_attr(attr.f) }
    }

    pub const fn set_subscr(self, subscr: Subscr) -> Self {
         self.set_slot_subscr(subscr.f)
    }
}

macro_rules! impl_slot_setter {
    ($(#[$attr:meta])* $fn_name:ident, $slot:expr, $ty:ty) => {
        impl ObjFullType {
            $(#[$attr])*
            pub const fn $fn_name(mut self, value: $ty) -> Self {
                *self.slot_index($slot) = $slot as u8;
                self.slots[$slot as usize - 1] = value as *const c_void;
                self
            }
        }
    };

    ($(#[$attr:meta])* unsafe $fn_name:ident, $slot:expr, $ty:ty) => {
        impl ObjFullType {
            $(#[$attr])*
            pub const unsafe fn $fn_name(mut self, value: $ty) -> Self {
                *self.slot_index($slot) = $slot as u8;
                self.slots[$slot as usize - 1] = value as *const c_void;
                self
            }
        }
    };
}

impl_slot_setter!(set_slot_unary_op, Slot::UnaryOp, UnaryOpFn);
impl_slot_setter!(set_slot_binary_op, Slot::BinaryOp, BinaryOpFn);
impl_slot_setter!(set_slot_subscr, Slot::Subscr, SubscrFn);

impl_slot_setter!(unsafe set_slot_make_new, Slot::MakeNew, MakeNewFn);
impl_slot_setter!(unsafe set_slot_attr, Slot::Attr, AttrFn);
impl_slot_setter!(unsafe set_slot_print, Slot::Print, PrintFn);
impl_slot_setter!(unsafe set_slot_locals_dict, Slot::LocalsDict, *mut Dict);
impl_slot_setter!(unsafe set_slot_protocol, Slot::Protocol, *const c_void);
impl_slot_setter!(unsafe set_slot_iter, Slot::Iter, *const c_void);

unsafe impl Sync for ObjFullType {}
unsafe impl Sync for ObjBase<'_> {}
// these are definitely not true but i dont car :)
unsafe impl Sync for Obj {}
unsafe impl Send for Obj {}

unsafe extern "C" {
    static mp_type_type: ObjType;
}

unsafe impl ObjTrait for ObjType {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_type };
}

unsafe impl ObjTrait for ObjFullType {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_type };
}

impl<'a> ObjBase<'a> {
    pub const fn new(ty: &'a ObjType) -> Self {
        Self {
            r#type: ty,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, Error)]
#[error("gc allocation failed")]
pub struct GcError;

impl Obj {
    pub const NULL: Self = unsafe { Self::from_ptr(core::ptr::null_mut()) };
    pub const SENTINEL: Self = unsafe { Self::from_ptr(4 as *mut c_void) };

    pub const NONE: Self = Self::from_immediate(0);
    pub const TRUE: Self = Self::from_immediate(3);
    pub const FALSE: Self = Self::from_immediate(1);

    pub fn new<T: ObjTrait>(o: T, alloc: &mut Gc) -> Result<Self, GcError> {
        unsafe {
            let mem = alloc.alloc(size_of::<T>());
            if mem.is_null() {
                return Err(GcError);
            }
            (mem as *mut T).write(o);
            Ok(Obj(mem as *mut c_void))
        }
    }

    pub const fn from_static<T: ObjTrait>(o: &'static T) -> Self {
        Self(o as *const T as *mut c_void)
    }

    pub const unsafe fn from_raw(inner: u32) -> Self {
        Self(inner as *mut c_void)
    }

    pub const fn from_int(int: i32) -> Self {
        // TODO: add overflow assertion
        Self(repr_c::new_int(int))
    }

    pub const fn from_qstr(qstr: Qstr) -> Self {
        Self(repr_c::new_qstr(qstr.index() as u32))
    }

    pub const fn from_immediate(imm: u32) -> Self {
        Self(repr_c::new_immediate(imm))
    }

    pub const fn from_bool(bool: bool) -> Self {
        match bool {
            true => Self::TRUE,
            false => Self::FALSE,
        }
    }

    pub const fn from_float(float: f32) -> Self {
        Self(repr_c::new_float(float))
    }

    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    pub fn is_int(self) -> bool {
        repr_c::is_int(self.0)
    }

    pub fn is_qstr(self) -> bool {
        repr_c::is_qstr(self.0)
    }

    pub fn is_immediate(self) -> bool {
        repr_c::is_immediate(self.0)
    }

    pub fn is_float(self) -> bool {
        repr_c::is_float(self.0)
    }

    pub fn is_ptr(self) -> bool {
        repr_c::is_ptr(self.0)
    }

    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub fn is_sentinel(self) -> bool {
        self.0 == Self::SENTINEL.0
    }

    pub fn is_none(&self) -> bool {
        self.0 == Self::NONE.0
    }

    pub fn is_bool(&self) -> bool {
        self.try_to_bool().is_some()
    }

    pub fn is(self, ty: &ObjType) -> bool {
        self.obj_type().map(|t| ty == t).unwrap_or(false)
    }

    pub fn ty(self) -> Option<repr_c::Ty> {
        repr_c::type_of(self.0)
    }

    pub fn try_to_int(self) -> Option<i32> {
        if repr_c::is_int(self.0) {
            Some(repr_c::get_int(self.0))
        } else {
            None
        }
    }

    pub fn try_to_qstr(self) -> Option<Qstr> {
        if repr_c::is_qstr(self.0) {
            Some(unsafe { Qstr::from_index(repr_c::get_qstr(self.0) as usize) })
        } else {
            None
        }
    }

    pub fn try_to_immediate(self) -> Option<u32> {
        if repr_c::is_immediate(self.0) {
            Some(repr_c::get_immediate(self.0))
        } else {
            None
        }
    }

    pub fn try_to_bool(self) -> Option<bool> {
        self.try_to_immediate().and_then(|imm| match imm {
            val if val == Self::TRUE.to_immediate() => Some(true),
            val if val == Self::FALSE.to_immediate() => Some(false),
            _ => None,
        })
    }

    pub fn try_to_float(self) -> Option<f32> {
        if repr_c::is_float(self.0) {
            Some(repr_c::get_float(self.0))
        } else {
            None
        }
    }

    pub fn try_to_obj_raw<T: ObjTrait>(self) -> Option<NonNull<T>> {
        if let Some(ty) = self.obj_type()
            && ty == T::OBJ_TYPE
        {
            Some(NonNull::new(self.0 as *mut T).unwrap())
        } else {
            None
        }
    }

    pub fn try_to_obj<T: ObjTrait>(&self) -> Option<&T> {
        self.try_to_obj_raw().map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn to_int(self) -> i32 {
        repr_c::get_int(self.0)
    }

    pub fn to_immediate(self) -> u32 {
        repr_c::get_immediate(self.0)
    }

    pub fn to_float(self) -> f32 {
        repr_c::get_float(self.0)
    }

    pub const fn inner(self) -> *mut c_void {
        self.0
    }

    pub fn get_str(&self) -> Option<&[u8]> {
        if let Some(qstr) = self.try_to_qstr() {
            return Some(qstr.bytes());
        }

        if let Some(str) = Self::try_to_obj::<Str>(self) {
            return Some(str.data());
        }

        None
    }

    // TODO: is this really static?
    pub fn obj_type(&self) -> Option<&ObjType> {
        if self.is_ptr() && !self.is_null() {
            let ptr = self.0 as *const ObjBase;
            Some(unsafe { &*(*ptr).r#type })
        } else {
            None
        }
    }
}

// for potential future use
//
// unsafe extern "C" {
//     fn mp_obj_print_helper(print: *const Print, o_in: Obj, kind: PrintKind);
// }
//
// pub fn print(&mut self, obj: Obj, kind: PrintKind) {
//     unsafe {
//         mp_obj_print_helper(&raw const mp_plat_print, obj, kind);
//     }
// }
