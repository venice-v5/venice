use std::task::{Context, Poll, Waker};
use std::{cell::RefCell, future::Future};
use std::pin::Pin;

use micropython_rs::obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags};
use vexide_devices::controller::ControllerScreenWriteFuture;

use crate::qstrgen::qstr;



pub static DEVICE_FUTURE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(DeviceFuture));

enum DeviceFuture {
    ControllerScreenWrite(ControllerScreenWriteFuture<'_>)
}

#[repr(C)]
pub struct DeviceFutureObj {
    base: ObjBase<'static>,
    pub future: RefCell<Pin<Box<dyn Future<Output = Obj>>>>,
}

unsafe impl ObjTrait for DeviceFutureObj {
    const OBJ_TYPE: &ObjType = DEVICE_FUTURE_OBJ_TYPE.as_obj_type();
}
