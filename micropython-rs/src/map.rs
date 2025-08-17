use crate::{
    obj::{Obj, ObjBase, ObjFullType, ObjType},
    qstr::Qstr,
};

unsafe extern "C" {
    /// From: `py/obj.h`
    fn mp_map_lookup(map: *mut Map, index: Obj, lookup_kind: LookupKind) -> *mut MapElem;
}

/// From: `py/obj.h`
#[derive(Clone, Copy)]
#[repr(C)]
struct MapElem {
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
    base: ObjBase,
    pub map: Map,
}

#[repr(C)]
pub struct ConstMapElem {
    key: Obj,
    value: Obj,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct ConstMap {
    // this is actually 4 bitfields
    used: usize,
    alloc: usize,
    table: *const ConstMapElem,
}

#[repr(C)]
pub struct ConstDict {
    base: ObjBase,
    pub map: ConstMap,
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
macro_rules! const_map {
    [$($key:expr => $value:expr),*] => {{
        use $crate::map::{ConstMap, ConstMapElem};

        static TABLE: &[ConstMapElem] = [$(ConstMapElem::new($key, $value)),*].as_slice();

        ConstMap::new(TABLE)
    }};
}

#[macro_export]
macro_rules! const_dict {
    [$($key:expr => $value:expr),*] => {{
        use $crate::{const_map, map::Dict};

        ConstDict::new(const_map![$($key => $value),*])
    }};
}

unsafe impl Sync for ConstMap {}

impl Map {
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

impl ConstMapElem {
    pub const fn new(qstr: Qstr, value: Obj) -> Self {
        Self {
            key: Obj::from_qstr(qstr),
            value,
        }
    }
}

impl ConstMap {
    pub const fn new(table: &'static [ConstMapElem]) -> Self {
        ConstMap {
            used: table.len() << 3 | 0b111,
            alloc: table.len(),
            table: table.as_ptr(),
        }
    }
}

unsafe extern "C" {
    /// From: `py/obj.h
    static mp_type_dict: ObjFullType;
}

impl ConstDict {
    pub const fn new(map: ConstMap) -> Self {
        Self {
            base: ObjBase::new(unsafe { &mp_type_dict }),
            map,
        }
    }
}

unsafe impl ObjType for Dict {
    const TYPE_OBJ: *const ObjFullType = &raw const mp_type_dict;
}
