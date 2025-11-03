use std::ffi::c_void;

use bitflags::bitflags;

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
    base: ObjBase,

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
    base: ObjBase,

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
pub struct ObjBase {
    r#type: *const ObjType,
}

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
/// - `01111111 10000000 00000000 ssss0000` is an immediate object
/// - `seeeeeee ffffffff ffffffff ffffff10` is a 30-bit float
/// - `pppppppp pppppppp pppppppp pppppp00` is a pointer to an object, starting with [`ObjBase`]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Obj(*mut c_void);

/// # Safety
///
/// Object representation must begin with an [`mp_obj_base_t`], always initialized to `OBJ_TYPE`.
/// Additionally, all instances must be aligned to 4 bytes in memory, but this is already
/// guaranteed if the first invariant is true, as a side effect.
pub unsafe trait ObjTrait: Sized {
    const OBJ_TYPE: *const ObjType;
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
    extern "C" fn(ty: *const ObjType, n_args: usize, n_kw: usize, args: *const Obj) -> Obj;
pub type PrintFn = extern "C" fn(print: *const Print, o: Obj, kind: PrintKind);
pub type CallFn = extern "C" fn(fun: Obj, n_args: usize, n_kw: usize, args: *const Obj) -> Obj;
pub type UnaryOpFn = extern "C" fn(op: UnaryOp, obj: Obj) -> Obj;
pub type BinaryOpFn = extern "C" fn(op: BinaryOp, obj: Obj) -> Obj;
pub type AttrFn = extern "C" fn(self_in: Obj, attr: Qstr, dest: *mut Obj);
pub type SubscrFn = extern "C" fn(self_in: Obj, index: Obj, value: Obj) -> Obj;

impl ObjFullType {
    pub const fn new(flags: TypeFlags, name: Qstr) -> Self {
        unsafe extern "C" {
            static mp_type_type: ObjType;
        }

        Self {
            base: unsafe { ObjBase::from_obj_type(&raw const mp_type_type) },
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

    pub const fn as_obj_type_ptr(&'static self) -> *const ObjType {
        self as *const Self as *const ObjType
    }

    pub const unsafe fn set_slot_locals_dict(mut self, value: *mut Dict) -> Self {
        let slot = Slot::LocalsDict;
        *self.slot_index(slot) = slot as u8;
        self.slots[slot as usize - 1] = value as *const c_void;
        self
    }

    pub const fn set_slot_locals_dict_from_static(self, value: &'static Dict) -> Self {
        unsafe { self.set_slot_locals_dict(value as *const Dict as *mut Dict) }
    }
}

macro_rules! impl_slot_setter {
    ($fn_name:ident, $slot:expr, $ty:ty) => {
        impl ObjFullType {
            pub const fn $fn_name(mut self, value: $ty) -> Self {
                *self.slot_index($slot) = $slot as u8;
                self.slots[$slot as usize - 1] = value as *const c_void;
                self
            }
        }
    };
}

impl_slot_setter!(set_slot_make_new, Slot::MakeNew, MakeNewFn);
impl_slot_setter!(set_slot_print, Slot::Print, PrintFn);
impl_slot_setter!(set_slot_unary_op, Slot::UnaryOp, UnaryOpFn);
impl_slot_setter!(set_slot_binary_op, Slot::BinaryOp, BinaryOpFn);
impl_slot_setter!(set_slot_attr, Slot::Attr, AttrFn);
impl_slot_setter!(set_slot_subscr, Slot::Subscr, SubscrFn);
impl_slot_setter!(set_slot_iter, Slot::Iter, *const c_void);
impl_slot_setter!(set_slot_protocol, Slot::Protocol, *const c_void);
impl_slot_setter!(set_slot_parent, Slot::Parent, *const c_void);

unsafe impl Sync for ObjFullType {}
unsafe impl Sync for ObjBase {}
// these are definitely not true but i dont car :)
unsafe impl Sync for Obj {}
unsafe impl Send for Obj {}

unsafe extern "C" {
    static mp_type_type: ObjType;
}

unsafe impl ObjTrait for ObjType {
    const OBJ_TYPE: *const ObjType = &raw const mp_type_type;
}

unsafe impl ObjTrait for ObjFullType {
    const OBJ_TYPE: *const ObjType = &raw const mp_type_type;
}

impl ObjBase {
    pub const fn new<O: ObjTrait>() -> Self {
        Self {
            r#type: O::OBJ_TYPE,
        }
    }

    pub const unsafe fn from_obj_type(r#type: *const ObjType) -> Self {
        Self { r#type }
    }
}

impl Obj {
    pub const NULL: Self = unsafe { Self::from_ptr(core::ptr::null_mut()) };
    pub const NONE: Self = Self::from_immediate(0);
    pub const TRUE: Self = Self::from_immediate(3);
    pub const FALSE: Self = Self::from_immediate(1);

    // TODO: return Result instead of Option
    pub fn new<T: ObjTrait>(o: T, alloc: &mut Gc) -> Option<Self> {
        unsafe {
            let mem = alloc.alloc(size_of::<T>());
            if mem.is_null() {
                return None;
            }
            (mem as *mut T).write(o);
            Some(Obj(mem as *mut c_void))
        }
    }

    pub const fn from_static<T: ObjTrait>(o: &'static T) -> Self {
        Self(o as *const T as *mut c_void)
    }

    pub const unsafe fn from_raw(inner: u32) -> Self {
        Self(inner as *mut c_void)
    }

    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    pub const fn from_small_int(int: i32) -> Self {
        // TODO: add overflow assertion
        unsafe { Self::from_raw((int << 1 | 0b1) as u32) }
    }

    pub const fn from_immediate(imm: u32) -> Self {
        unsafe { Self::from_raw(imm << 3 | 0b110) }
    }

    pub const fn from_qstr(qstr: Qstr) -> Self {
        unsafe { Self::from_raw((qstr.index() as u32) << 3 | 0b010) }
    }

    pub fn as_small_int(self) -> Option<i32> {
        let int = self.0 as i32;
        if int & 0b1 != 1 {
            return None;
        }
        // right shifting a signed integer (as opposed to an unsigned int) performs an arithmetic
        // right shift where the sign bit is preserved, e.g. 0b1000 >> 1 = 0b1100
        Some(int >> 1)
    }

    pub const fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn is_none(&self) -> bool {
        self.0 == Self::NONE.0
    }

    pub fn as_qstr(&self) -> Option<Qstr> {
        if self.0 as u32 & 0b111 == 0b10 {
            Some(unsafe { Qstr::from_index((self.0 as u32 >> 3) as usize) })
        } else {
            None
        }
    }

    pub fn get_str(&self) -> Option<&[u8]> {
        if let Some(qstr) = self.as_qstr() {
            return Some(qstr.bytes());
        }

        if let Some(str) = Self::as_obj::<Str>(self) {
            return Some(str.data());
        }

        None
    }

    pub fn obj_type(&self) -> Option<*const ObjType> {
        if self.0 as u32 & 0b11 != 0 {
            let ptr = self.0 as *const ObjBase;
            Some(unsafe { *ptr }.r#type)
        } else {
            None
        }
    }

    pub fn is(&self, ty: *const ObjType) -> bool {
        self.obj_type().map(|t| ty == t).unwrap_or(false)
    }

    pub const fn as_ptr(&self) -> *mut c_void {
        self.0
    }

    pub fn as_obj_raw<T: ObjTrait>(&self) -> Option<*mut T> {
        if self.0 as u32 & 0b11 != 0 {
            return None;
        }

        let ptr = self.0 as *mut ObjBase;
        if unsafe { *ptr }.r#type != T::OBJ_TYPE {
            return None;
        }

        Some(ptr as *mut T)
    }

    pub fn as_obj<T: ObjTrait>(&self) -> Option<&T> {
        self.as_obj_raw().map(|ptr| unsafe { &*ptr })
    }

    pub fn as_immediate(&self) -> Option<u32> {
        if self.0 as u32 & 0b111 != 0b110 {
            None
        } else {
            Some(self.0 as u32 >> 3)
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.as_immediate().and_then(|imm| match imm {
            3 => Some(true),
            1 => Some(false),
            _ => None,
        })
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
