use std::{
    cell::RefCell,
    ffi::c_int,
    io::{Read, Write},
    pin::Pin,
    task::{Context, Waker},
};

use micropython_rs::{
    const_dict,
    errno::{MP_EINVAL, MP_EIO},
    except::{raise_stop_iteration, raise_value_error},
    init::token,
    ioctl_from_fn, make_new_from_fn,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    read_from_fn,
    stream::{IoctlReq, Poll, Stream},
    write_from_fn,
};
use vexide_devices::smart::serial::{SerialPort, SerialPortOpenFuture};

use crate::{
    args::Args,
    devices::{PortNumber, lock_port},
    modvenice::vasyncio::event_loop::WAKE_SIGNAL,
    obj::alloc_obj,
    qstrgen::qstr,
    registry::{RegistryGuard, UpgradeGuard},
};

#[repr(C)]
pub struct SerialPortObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, SerialPort>,
}

#[repr(C)]
pub struct SerialPortOpenFutureObj {
    base: ObjBase<'static>,
    upgrade: RefCell<Option<UpgradeGuard<'static, SerialPortOpenFuture>>>,
}

pub static SERIAL_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(SerialPort))
    .set_make_new(make_new_from_fn!(serial_make_new))
    .set_stream(&SERIAL_STREAM)
    .set_locals_dict(const_dict![]); // TODO

pub static SERIAL_FUTURE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(SerialPortOpenFuture))
        .set_iter(Iter::IterNext(serial_future_iternext));

unsafe impl ObjTrait for SerialPortObj {
    const OBJ_TYPE: &ObjType = SERIAL_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for SerialPortOpenFutureObj {
    const OBJ_TYPE: &ObjType = SERIAL_FUTURE_OBJ_TYPE.as_obj_type();
}

fn serial_make_new(_: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, c"port number must be between 1 and 21"));
    let baud_rate = reader.next_positional::<i32>() as u32;

    let upgrade = lock_port(port, |p| p)
        .start_upgrade()
        .unwrap()
        .map(|p| SerialPort::open(p, baud_rate));

    alloc_obj(SerialPortOpenFutureObj {
        base: ObjBase::new(SerialPortOpenFutureObj::OBJ_TYPE),
        upgrade: RefCell::new(Some(upgrade)),
    })
}

extern "C" fn serial_future_iternext(self_in: Obj) -> Obj {
    let this = self_in.try_as_obj::<SerialPortOpenFutureObj>().unwrap();
    let mut refmut = this.upgrade.borrow_mut();
    let Some(mut upgrade) = refmut.take() else {
        raise_stop_iteration(token(), Obj::NONE)
    };

    let future = upgrade.as_mut();

    let mut cx = Context::from_waker(Waker::noop());
    match Future::poll(Pin::new(future), &mut cx) {
        std::task::Poll::Ready(serial_port) => {
            let guard = RegistryGuard::finish_upgrade(upgrade.map(|_| serial_port));
            raise_stop_iteration(
                token(),
                alloc_obj(SerialPortObj {
                    base: ObjBase::new(SerialPortObj::OBJ_TYPE),
                    guard,
                }),
            );
        }
        std::task::Poll::Pending => {
            *refmut = Some(upgrade);
            Obj::from_static(&WAKE_SIGNAL)
        }
    }
}

fn err_to_code(_err: std::io::Error) -> c_int {
    // vexide always returns non-os io errors, so don't bother to check using `raw_os_error`
    MP_EIO
}

fn serial_read(self_in: Obj, buf: &mut [u8]) -> Result<usize, c_int> {
    self_in
        .try_as_obj::<SerialPortObj>()
        .unwrap()
        .guard
        .borrow_mut()
        .read(buf)
        .map_err(err_to_code)
}

fn serial_write(self_in: Obj, buf: &[u8]) -> Result<usize, c_int> {
    self_in
        .try_as_obj::<SerialPortObj>()
        .unwrap()
        .guard
        .borrow_mut()
        .write(buf)
        .map_err(err_to_code)
}

fn serial_ioctl(self_in: Obj, req: IoctlReq) -> Result<usize, c_int> {
    let this = self_in.try_as_obj::<SerialPortObj>().unwrap();
    let mut serial = this.guard.borrow_mut();

    match req {
        IoctlReq::Poll(poll) => {
            let mut ret = Poll::empty();

            if poll.contains(Poll::RD) {
                ret |= serial
                    .unread_bytes()
                    .map(|b| if b > 0 { Poll::RD } else { Poll::empty() })
                    .unwrap_or(Poll::ERR);
            }

            if poll.contains(Poll::WR) {
                ret |= serial
                    .write_capacity()
                    .map(|b| if b > 0 { Poll::WR } else { Poll::empty() })
                    .unwrap_or(Poll::ERR);
            }

            Ok(ret.bits())
        }
        IoctlReq::Flush => {
            serial.flush().map_err(err_to_code)?;
            Ok(0)
        }
        _ => Err(MP_EINVAL),
    }
}

const SERIAL_STREAM: Stream = Stream {
    read: read_from_fn!(serial_read),
    write: write_from_fn!(serial_write),
    ioctl: ioctl_from_fn!(serial_ioctl),
    is_text: 1, // uhh maybe
};
