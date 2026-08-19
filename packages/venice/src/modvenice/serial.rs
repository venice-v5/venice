use std::{
    cell::RefCell,
    ffi::c_int,
    io::{Read, Write},
    pin::Pin,
    task::{Context, Waker},
};

use micropython_macros::{class, class_methods};
use micropython_rs::{
    errno::{MP_EINVAL, MP_EIO},
    except::{raise_stop_iteration, runtime_error, value_error},
    fun::{Fun1, FunVarBetween},
    init::token,
    ioctl_from_fn,
    obj::{Obj, ObjBase, ObjTrait},
    read_from_fn,
    stream::{
        IoctlReq, Poll, Stream, mp_stream_flush_obj, mp_stream_ioctl_obj, mp_stream_read_obj,
        mp_stream_read1_obj, mp_stream_write_obj, mp_stream_write1_obj,
    },
    write_from_fn,
};
use vexide_devices::smart::{
    SmartPort,
    serial::{SerialPort, SerialPortOpenFuture},
};

use crate::{
    devices::{PortNumber, lock_port},
    obj::alloc_obj,
    registry::{RegistryGuard, SmartGuard, UpgradeGuard},
};

fn checked_baud_rate(baud_rate: i32) -> Option<u32> {
    (1..=SerialPort::MAX_BAUD_RATE as i32)
        .contains(&baud_rate)
        .then_some(baud_rate as u32)
}

/// A Smart Port configured as a generic RS-485 serial port.
///
/// This class provides an interface for using V5 Smart Ports as serial communication ports over
/// RS-485. It allows bidirectional communication with any device that speaks serial over the V5's
/// RS-485 interface.
///
/// # Hardware Description
///
/// V5 Smart Ports provide half-duplex RS-485 serial communication at up to an allowed 921600 baud for
/// user programs.
///
/// The ports supply 12.8V VCC nominally (VCC is wired directly to the V5's battery lines, providing
/// voltage somewhere in the range of 12-14V). Writes to the serial port are buffered, but are
/// automatically flushed by VEXos as fast as possible (down to ~10µs or so).
///
/// Open a port with `await SerialPort.open(...)`; `SerialPort` cannot be constructed directly. The
/// object implements the MicroPython stream methods `read`, `read1`, `write`, `write1`, `flush`, and
/// `ioctl`. Reads consume the bytes currently available rather than waiting for the requested
/// amount and return `bytes`. After `free` releases the Smart Port, stream and device operations
/// raise `ValueError`.
#[class(qstr!(SerialPort))]
#[repr(C)]
pub struct SerialPortObj {
    base: ObjBase,
    guard: SmartGuard<SerialPort>,
}

/// An awaitable that opens and configures a `SerialPort`.
///
/// If the port was not previously configured as a generic serial port, this may take a few milliseconds
/// to complete. Awaiting this object returns the opened `SerialPort`. Users receive it from
/// `SerialPort.open`; it cannot be constructed directly. Awaiting the same instance more than once
/// raises `RuntimeError`.
#[class(qstr!(SerialPortOpenFuture))]
#[repr(C)]
pub struct SerialPortOpenFutureObj {
    base: ObjBase,
    upgrade: RefCell<Option<UpgradeGuard<'static, SmartPort, SerialPortOpenFuture>>>,
}

#[class_methods]
impl SerialPortObj {
    /// The length of the serial FIFO input and output buffers, in bytes. Its value is 1024.
    #[constant]
    const INTERNAL_BUFFER_SIZE: i32 = SerialPort::INTERNAL_BUFFER_SIZE as i32;

    /// The maximum user-configurable baud rate for generic serial under normal conditions. Its value is 921600.
    #[constant]
    const MAX_BAUD_RATE: i32 = SerialPort::MAX_BAUD_RATE as i32;

    /// Opens and configures a generic serial port on Smart Port `port_number`.
    ///
    /// This configures a Smart Port to act as a generic serial controller capable of sending and receiving
    /// data. Providing `baud_rate`, or the transmission rate of bits, is required. It must be from
    /// 1 through `SerialPort.MAX_BAUD_RATE` (921600). Await the returned `SerialPortOpenFuture` to
    /// obtain the usable port.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     serial = await SerialPort.open(1, 115200)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port_number` is outside 1 through 21, its Smart Port is occupied, or
    ///   `baud_rate` is outside 1 through `SerialPort.MAX_BAUD_RATE`.
    /// - `TypeError`: If either argument is not an integer.
    #[method(binding = "static")]
    #[stub(sig = "(port_number: int, baud_rate: int, /) -> SerialPortOpenFuture")]
    fn open(
        port_number: PortNumber,
        baud_rate: i32,
    ) -> Result<SerialPortOpenFutureObj, crate::modvenice::Exception> {
        let baud_rate = checked_baud_rate(baud_rate).ok_or_else(|| {
            value_error(c"baud_rate must be between 1 and SerialPort.MAX_BAUD_RATE")
        })?;
        let upgrade = lock_port(port_number, |p| p)
            .start_upgrade()
            .unwrap()
            .map(|p| SerialPort::open(p, baud_rate));

        Ok(SerialPortOpenFutureObj {
            base: ObjBase::new(SerialPortOpenFutureObj::OBJ_TYPE),
            upgrade: RefCell::new(Some(upgrade)),
        })
    }

    /// Configures the baud rate of the serial port.
    ///
    /// Baud rate determines the speed of communication over the data channel. It must be from 1
    /// through `SerialPort.MAX_BAUD_RATE` (921600).
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     serial = await SerialPort.open(1, 115200)
    ///
    ///     # Change to 9600 baud.
    ///     serial.set_baud_rate(9600)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `baud_rate` is not an integer.
    /// - `ValueError`: If `baud_rate` is outside 1 through `SerialPort.MAX_BAUD_RATE`, or the port
    ///   has been freed.
    #[method]
    fn set_baud_rate(&self, baud_rate: i32) -> Result<(), crate::modvenice::Exception> {
        let baud_rate = checked_baud_rate(baud_rate).ok_or_else(|| {
            value_error(c"baud_rate must be between 1 and SerialPort.MAX_BAUD_RATE")
        })?;
        self.guard.borrow_mut().set_baud_rate(baud_rate);
        Ok(())
    }

    /// Clears the internal input and output FIFO buffers.
    ///
    /// This can be useful to reset state and remove old, potentially unneeded data from the input FIFO
    /// buffer or to cancel sending any data in the output FIFO buffer.
    ///
    /// # This is not the same thing as "flushing".
    ///
    /// This method does not cause the data in the output buffer to be written. It simply clears the
    /// internal buffers. Unlike standard output, generic serial does not use buffered I/O (the FIFO
    /// buffers are written as soon as possible).
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     serial = await SerialPort.open(1, 115200)
    ///
    ///     serial.clear_buffers()
    ///     serial.write(b"Buffers are clear!")
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the port has been freed.
    #[method]
    fn clear_buffers(&self) {
        self.guard.borrow_mut().clear_buffers();
    }

    /// Releases this binding's Smart Port lock and makes the object unusable.
    ///
    /// The port number can then be used to construct another device.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the port has already been freed.
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

    /// Provides MicroPython readable, writable, flush, and poll stream behavior.
    ///
    /// Stream I/O is immediate against the VEXos FIFO buffers. Internal serial failures are reported as
    /// `OSError` with error code `EIO`.
    #[stream]
    const STREAM: Stream = Stream {
        read: read_from_fn!(SerialPortObj::stream_read),
        write: write_from_fn!(SerialPortObj::stream_write),
        ioctl: ioctl_from_fn!(SerialPortObj::stream_ioctl),
        is_text: 0,
    };

    /// Reads up to `size` currently available bytes from the serial port's FIFO input buffer.
    ///
    /// `size=-1`, the default, drains all data currently available. A nonnegative `size` limits the result;
    /// values below -1 are invalid MicroPython stream sizes. An empty FIFO returns `b""` without
    /// waiting.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `size` is not an integer.
    /// - `MemoryError`: If `size` is below -1 or is too large to allocate the result.
    /// - `OSError`: If the serial read fails.
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(read))]
    #[stub(sig = "(self, size: int = -1, /) -> bytes")]
    const READ: &FunVarBetween = &mp_stream_read_obj;

    /// Reads up to `size` currently available bytes from the serial port's FIFO input buffer with
    /// single-read semantics.
    ///
    /// For a nonnegative `size`, MicroPython performs at most one low-level read. `size=-1`, the default,
    /// instead drains all data currently available. An empty FIFO returns `b""`.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `size` is not an integer.
    /// - `MemoryError`: If `size` is below -1 or is too large to allocate the result.
    /// - `OSError`: If the serial read fails.
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(read1))]
    #[stub(sig = "(self, size: int = -1, /) -> bytes")]
    const READ1: &FunVarBetween = &mp_stream_read1_obj;

    /// Writes `buffer` to the serial port's FIFO output buffer and returns the number of bytes accepted.
    ///
    /// `buffer` must provide a readable buffer, such as `bytes`, `bytearray`, or `memoryview`. MicroPython
    /// repeats low-level writes while progress is made, so this method attempts to enqueue the complete
    /// buffer.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `buffer` does not support the readable buffer protocol.
    /// - `OSError`: If the serial write fails.
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(write))]
    #[stub(sig = "(self, buffer: bytes | bytearray | memoryview, /) -> int")]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;

    /// Performs one low-level write from `buffer` to the serial port's FIFO output buffer and returns the
    /// number of bytes accepted.
    ///
    /// This can return fewer bytes than the length of `buffer` when the output FIFO lacks space. `buffer`
    /// must be `bytes`, `bytearray`, `memoryview`, or another readable buffer object.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `buffer` does not support the readable buffer protocol.
    /// - `OSError`: If the serial write fails.
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(write1))]
    #[stub(sig = "(self, buffer: bytes | bytearray | memoryview, /) -> int")]
    const WRITE1: &FunVarBetween = &mp_stream_write1_obj;

    /// Completes the MicroPython flush operation and returns `None`.
    ///
    /// The underlying generic-serial flush is a no-op because VEXos already transmits queued output as
    /// quickly as possible. This method does not call `SerialPort.clear_buffers`.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(flush))]
    #[stub(sig = "(self, /) -> None")]
    const FLUSH: &Fun1 = &mp_stream_flush_obj;

    /// Performs a low-level MicroPython stream control `request` with integer `arg`, which defaults
    /// to `0`.
    ///
    /// Request `1` flushes the stream. Request `3` polls the flags in `arg`: readable is `0x01`, writable
    /// is `0x04`, and an I/O error is returned as `0x08`. Other requests are unsupported.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `request` or `arg` is not an integer.
    /// - `OSError`: If `request` is unsupported.
    /// - `ValueError`: If the port has been freed.
    #[constant(qstr!(ioctl))]
    #[stub(sig = "(self, request: int, arg: int = 0, /) -> int")]
    const IOCTL: &FunVarBetween = &mp_stream_ioctl_obj;
}

#[class_methods]
impl SerialPortOpenFutureObj {
    /// Advances the one-shot awaitable.
    ///
    /// Returns the opened `SerialPort` when configuration finishes and otherwise yields control to the
    /// scheduler. Users should use `await` rather than calling this protocol operation directly.
    ///
    /// # Raises
    ///
    /// - `RuntimeError`: If the same one-shot future is awaited more than once.
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
        let this = self_in.try_as_obj::<SerialPortOpenFutureObj>().unwrap();
        let upgrade = this.upgrade.borrow_mut().take();
        let Some(mut upgrade) = upgrade else {
            runtime_error(c"SerialPortOpenFuture cannot be awaited more than once").raise(token())
        };

        let mut cx = Context::from_waker(Waker::noop());
        match Future::poll(Pin::new(upgrade.as_mut()), &mut cx) {
            std::task::Poll::Ready(serial_port) => {
                let guard = RegistryGuard::finish_upgrade(upgrade.map(|_| serial_port));
                let port = alloc_obj(SerialPortObj {
                    base: ObjBase::new(SerialPortObj::OBJ_TYPE),
                    guard,
                });
                raise_stop_iteration(token(), port);
            }
            std::task::Poll::Pending => {
                *this.upgrade.borrow_mut() = Some(upgrade);
                Obj::NONE
            }
        }
    }
}

pub fn err_to_code(_err: std::io::Error) -> c_int {
    // vexide always returns non-os io errors, so don't bother to check using `raw_os_error`
    MP_EIO
}
