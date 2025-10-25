use alloc::collections::binary_heap::BinaryHeap;

use micropython_rs::obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags};

use crate::{ALLOCATOR, qstrgen::qstr};

pub static EVENT_LOOP_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(EventLoop));

#[repr(C)]
pub struct EventLoop {
    base: ObjBase,
    tasks: BinaryHeap<()>,
}

unsafe impl ObjTrait for EventLoop {
    const OBJ_TYPE: *const ObjType = &raw const EVENT_LOOP_TYPE as *const ObjType;
}

pub extern "C" fn new_event_loop() -> Obj {
    let eloop = EventLoop {
        base: ObjBase::new::<EventLoop>(),
        tasks: BinaryHeap::new(),
    };

    Obj::new(eloop, ALLOCATOR.lock().as_mut().unwrap()).unwrap()
}
