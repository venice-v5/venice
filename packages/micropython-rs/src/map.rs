use crate::obj::{Obj, ObjBase, ObjTrait, ObjType};

unsafe extern "C" {
    /// From: `py/obj.h`
    fn mp_map_lookup(map: *mut Map, index: Obj, lookup_kind: LookupKind) -> *mut MapElem;
}

/// From: `py/obj.h`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapElem {
    pub key: Obj,
    pub value: Obj,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct Map {
    // this is actually 4 bitfields
    used: usize,
    alloc: usize,
    table: *mut MapElem,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct Dict {
    base: ObjBase<'static>,
    pub map: Map,
}

/// From: `py/obj.h`
#[allow(dead_code)]
#[repr(C)]
enum LookupKind {
    Lookup = 0,
    AddIfNotFound = 1,
    RemoveIfFound = 2,
    // the real name is longer than this
    AddIfNotFoundOrRemoveIfFound = 3,
}

#[macro_export]
macro_rules! map_table {
    [$($key:expr => $value:expr),* $(,)?] => {
        [$($crate::map::MapElem {
            key: $crate::obj::Obj::from_qstr($key),
            value: $value,
        }),*]
    }
}

#[macro_export]
macro_rules! const_map {
    [$($key:expr => $value:expr),* $(,)?] => {{
        static mut TABLE: &[$crate::map::MapElem] = [$($crate::map::MapElem {
            key: $crate::obj::Obj::from_qstr($key),
            value: $value,
        }),*].as_slice();

        unsafe {
            $crate::map::Map::from_raw_parts(TABLE.as_ptr() as *mut $crate::map::MapElem, TABLE.len(), TABLE.len(), true, true, true)
        }
    }};
}

#[macro_export]
macro_rules! const_dict {
    [$($key:expr => $value:expr),* $(,)?] => {{
        static mut DICT: $crate::map::Dict = $crate::map::Dict::new($crate::const_map![$($key => $value),*]);
        unsafe { &DICT }
    }};
}

impl Map {
    pub const unsafe fn from_raw_parts(
        ptr: *mut MapElem,
        len: usize,
        alloc: usize,
        all_qstr_keys: bool,
        fixed: bool,
        ordered: bool,
    ) -> Self {
        Self {
            used: len << 3
                | ((ordered as usize) << 2)
                | ((fixed as usize) << 1)
                | (all_qstr_keys as usize),
            alloc,
            table: ptr,
        }
    }

    pub fn is_fixed(&self) -> bool {
        self.used & 0b10 != 0
    }

    pub fn get(&self, index: Obj) -> Option<Obj> {
        unsafe {
            let elem = mp_map_lookup(self as *const Self as *mut Self, index, LookupKind::Lookup);

            if elem.is_null() {
                None
            } else {
                Some((*elem).value)
            }
        }
    }

    pub fn insert(&mut self, index: Obj, value: Obj) -> Obj {
        unsafe {
            let elem = mp_map_lookup(self as *mut Self, index, LookupKind::AddIfNotFound);
            let old = (*elem).value;
            (*elem).value = value;
            old
        }
    }

    pub fn remove(&mut self, index: Obj) -> Option<Obj> {
        unsafe {
            let elem = mp_map_lookup(self as *mut Self, index, LookupKind::RemoveIfFound);

            if elem.is_null() {
                None
            } else {
                Some((*elem).value)
            }
        }
    }
}

unsafe extern "C" {
    /// From: `py/obj.h
    static mp_type_dict: ObjType;
}

impl Dict {
    pub const fn new(map: Map) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            map,
        }
    }
}

unsafe impl ObjTrait for Dict {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_dict };
}
