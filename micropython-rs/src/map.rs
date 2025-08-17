use crate::obj::{Obj, ObjBase, ObjFullType, ObjType};

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

unsafe extern "C" {
    /// From: `py/obj.h
    static mp_type_dict: ObjFullType;
}

unsafe impl ObjType for Dict {
    const TYPE_OBJ: *const ObjFullType = &raw const mp_type_dict;
}
