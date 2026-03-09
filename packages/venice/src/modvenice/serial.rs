use std::{
    cell::RefCell,
    ffi::c_int,
    io::{Read, Write},
    pin::Pin,
    task::{Context, Waker},
};

use micropython_rs::{
    class, class_methods,
    errno::{MP_EINVAL, MP_EIO},
    except::{raise_stop_iteration, raise_value_error},
    fun::{Fun2, FunVarBetween},
    init::token,
    ioctl_from_fn,
    obj::{Obj, ObjBase, ObjTrait},
    read_from_fn,
    stream::{
        IoctlReq, Poll, Stream, mp_stream_ioctl_obj, mp_stream_read_obj, mp_stream_read1_obj,
        mp_stream_write_obj, mp_stream_write1_obj,
    },
    write_from_fn,
};
use vexide_devices::smart::serial::{SerialPort, SerialPortOpenFuture};

use crate::{
    devices::{PortNumber, lock_port},
    modvenice::vasyncio::event_loop::WAKE_SIGNAL,
    obj::alloc_obj,
    qstrgen::qstr,
    registry::{RegistryGuard, UpgradeGuard},
};

#[class(qstr!(SerialPort))]
#[repr(C)]
pub struct SerialPortObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, SerialPort>,
}

#[class(qstr!(SerialPortOpenFuture))]
#[repr(C)]
pub struct SerialPortOpenFutureObj {
    base: ObjBase<'static>,
    upgrade: RefCell<Option<UpgradeGuard<'static, SerialPortOpenFuture>>>,
}

#[class_methods]
impl SerialPortObj {
    #[constant]
    const INTERNAL_BUFFER_SIZE: i32 = SerialPort::INTERNAL_BUFFER_SIZE as i32;
    #[constant]
    const MAX_BAUD_RATE: i32 = SerialPort::MAX_BAUD_RATE as i32;

    #[method(binding = "static")]
    fn open(port: i32, baud_rate: i32) -> SerialPortOpenFutureObj {
        let port_number = PortNumber::from_i32(port).unwrap_or_else(|_| {
            raise_value_error(token(), c"port number must be between 1 and 21")
        });

        let upgrade = lock_port(port_number, |p| p)
            .start_upgrade()
            .unwrap()
            .map(|p| SerialPort::open(p, baud_rate as u32));

        SerialPortOpenFutureObj {
            base: ObjBase::new(SerialPortOpenFutureObj::OBJ_TYPE),
            upgrade: RefCell::new(Some(upgrade)),
        }
    }

    #[method]
    fn set_baud_rate(&self, baud_rate: i32) {
        self.guard.borrow_mut().set_baud_rate(baud_rate as u32);
    }

    #[method]
    fn clear_buffers(&self) {
        self.guard.borrow_mut().clear_buffers();
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }

    fn stream_read(self_in: Obj, buf: &mut [u8]) -> Result<usize, c_int> {
        self_in
            .try_as_obj::<SerialPortObj>()
            .unwrap()
            .guard
            .borrow_mut()
            .read(buf)
            .map_err(err_to_code)
    }

    fn stream_write(self_in: Obj, buf: &[u8]) -> Result<usize, c_int> {
        self_in
            .try_as_obj::<SerialPortObj>()
            .unwrap()
            .guard
            .borrow_mut()
            .write(buf)
            .map_err(err_to_code)
    }

    fn stream_ioctl(self_in: Obj, req: IoctlReq) -> Result<usize, c_int> {
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

    #[stream]
    const STREAM: Stream = Stream {
        read: read_from_fn!(SerialPortObj::stream_read),
        write: write_from_fn!(SerialPortObj::stream_write),
        ioctl: ioctl_from_fn!(SerialPortObj::stream_ioctl),
        is_text: 1, // uhh maybe
    };

    #[constant(qstr!(read))]
    const READ: &FunVarBetween = &mp_stream_read_obj;
    #[constant(qstr!(read1))]
    const READ1: &FunVarBetween = &mp_stream_read1_obj;
    #[constant(qstr!(write))]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;
    #[constant(qstr!(write1))]
    const WRITE1: &FunVarBetween = &mp_stream_write_obj;
    #[constant(qstr!(flush))]
    const FLUSH: &Fun2 = &mp_stream_write1_obj;
    #[constant(qstr!(ioctl))]
    const IOCTL: &FunVarBetween = &mp_stream_ioctl_obj;
}

#[class_methods]
impl SerialPortOpenFutureObj {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
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
}

fn err_to_code(_err: std::io::Error) -> c_int {
    // vexide always returns non-os io errors, so don't bother to check using `raw_os_error`
    MP_EIO
}
