use micropython_rs::except::raise_stop_iteration;
use micropython_rs::init::token;
use micropython_rs::obj::{IterSlotValue, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags};

use crate::modvenice::controller::ControllerScreenWriteAwaitable;
use crate::modvenice::raise_device_error;
use crate::obj::alloc_obj;
use crate::qstrgen::qstr;



pub static DEVICE_FUTURE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(DeviceFuture)).set_iter(IterSlotValue::IterNext(device_future_iternext));

extern "C" fn device_future_iternext(self_in: Obj) -> Obj {
    let this = self_in.try_to_obj::<DeviceFutureObj>().unwrap();
    if let Some(out) = this.poll() {
        raise_stop_iteration(token().unwrap(), out);
    } else {
        self_in
    }
}

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
