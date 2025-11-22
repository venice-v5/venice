use std::task::{Context, Poll, Waker};
use std::{cell::RefCell, future::Future};
use std::pin::Pin;

use micropython_rs::init::token;
use micropython_rs::obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags};
use vexide_devices::controller::ControllerScreenWriteFuture;

use crate::modvenice::controller::ControllerScreenWriteAwaitable;
use crate::modvenice::raise_device_error;
use crate::obj::alloc_obj;
use crate::qstrgen::qstr;



pub static DEVICE_FUTURE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(DeviceFuture));

pub enum DeviceFuture {
    ControllerScreenWrite(ControllerScreenWriteAwaitable)
}

#[repr(C)]
pub struct DeviceFutureObj {
    base: ObjBase<'static>,
    future: DeviceFuture,
}

unsafe impl ObjTrait for DeviceFutureObj {
    const OBJ_TYPE: &ObjType = DEVICE_FUTURE_OBJ_TYPE.as_obj_type();
}

impl DeviceFutureObj {
    pub fn new(future: DeviceFuture) -> Obj {
        alloc_obj(Self {
            base: ObjBase::new(&Self::OBJ_TYPE),
            future
        })
    }
    pub fn poll(&self) -> Option<Obj> {
        match &self.future {
            DeviceFuture::ControllerScreenWrite(awaitable) => awaitable.poll().map(|res| {
                if let Err(e) = res { raise_device_error(token().unwrap(), format!("{e}")) }
                Obj::NONE
            })
        }
    }
}
