use std::{
    ffi::c_int,
    io::{Read, Write},
};

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    errno::MP_EINVAL,
    except::value_error,
    fun::{Fun1, Fun2, FunVarBetween},
    ioctl_from_fn,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
    print::{Print, PrintKind},
    read_from_fn,
    stream::{
        IoctlReq, Poll, Stream, mp_stream_flush_obj, mp_stream_ioctl_obj, mp_stream_read_obj,
        mp_stream_read1_obj, mp_stream_write_obj, mp_stream_write1_obj,
    },
    write_from_fn,
};
use vexide_devices::smart::link::{LinkType, RadioLink};

use crate::{
    devices,
    modvenice::{Exception, serial::err_to_code},
    registry::SmartGuard,
};

/// VEXlink WireLess Radio Link
///
/// This class provides support for VEXlink, a point-to-point wireless communications protocol
/// between two VEXnet radios.
///
/// # Hardware Overview
///
/// There are two types of radios in a VEXlink connection: "manager" and "worker". A "manager" radio
/// can transmit data at up to 1040 bytes/s while a "worker" radio can transmit data at up to 520
/// bytes/s. A connection should only ever have both types of radios.
///
/// In order to connect to a radio, VEXos hashes a given link name and uses it as an ID to verify
/// the connection. For this reason, you should try to create a unique name for each radio link to
/// avoid accidentally interfering, or being interfered with by, an unrelated VEXlink connection.
/// Ideally, you want a name that will never be used by another team.
///
/// The lights on the radio can be used as a status indicator:
/// - Blinking red: The radio is waiting for a connection to be established.
/// - Alternating red and green quickly: The radio is connected to another radio and is the
///   "manager" radio.
/// - Alternating red and green slowly: The radio is connected to another radio and is the "worker"
///   radio.
///
/// For further information, see <https://www.vexforum.com/t/vexlink-documentaton/84538>
#[class(qstr!(RadioLink))]
pub struct RadioLinkObj {
    base: ObjBase,
    guard: SmartGuard<RadioLink>,
}

/// The type of radio link being established.
///
/// VEXLink is a point-to-point connection, with one "manager" robot and one "worker" robot.
#[class(qstr!(LinkType))]
pub struct LinkTypeObj {
    base: ObjBase,
    ty: LinkType,
}

#[class_methods]
impl LinkTypeObj {
    const fn new(ty: LinkType) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            ty,
        }
    }

    /// Manager Radio
    ///
    /// This end of the link has a 1040-bytes/sec data rate when communicating with a worker radio.
    #[constant]
    const MANAGER: &Self = &Self::new(LinkType::Manager);

    /// Worker Radio
    ///
    /// This end of the link has a 520-bytes/sec data rate when communicating with a manager radio.
    #[constant]
    const WORKER: &Self = &Self::new(LinkType::Worker);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.ty {
            LinkType::Manager => "LinkType.MANAGER",
            LinkType::Worker => "LinkType.WORKER",
        });
    }
}

#[class_methods]
impl RadioLinkObj {
    /// The length of the link's FIFO input and output buffers.
    #[constant]
    const INTERNAL_BUFFER_SIZE: i32 = RadioLink::INTERNAL_BUFFER_SIZE as i32;

    /// Opens a radio link from a VEXNet radio plugged into a Smart Port. Once opened, other VEXNet
    /// functionality such as controller tethering on this specific radio will be disabled. Other
    /// radios connected to the Brain can take over this functionality.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If a NUL (0x00) character was found anywhere in the specified `id`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// link = RadioLink(1, "643A", LinkType.MANAGER)
    /// ```
    #[make_new]
    #[stub(sig = "(self, port: int, id: str, link_type: LinkType) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(3, 3).assert_nkw(0, 0);

        let port_number = reader.next_positional()?;
        let id = reader.next_positional::<&str>()?;
        if id.contains('\0') {
            Err(value_error(c"id must not contain a nul byte ('\\0')"))?
        }
        let link_type = reader.next_positional::<&LinkTypeObj>()?;

        Ok(Self {
            base: ty.into(),
            guard: devices::lock_port(port_number, |p| RadioLink::open(p, id, link_type.ty)),
        })
    }

    /// Returns `True` if there is a link established with another radio.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// link = RadioLink(1, "643A", LinkType.MANAGER)
    /// if link.is_linked():
    ///     link.write(b"Hello!")
    /// ```
    #[method]
    fn is_linked(&self) -> bool {
        self.guard.borrow_mut().is_linked()
    }

    /// Release this device and free its Smart Port lock. This binding will become unusable after
    /// this call, but you can reuse the underlying Smart Port number in a new device.
    ///
    /// Any attempts to use this device after freeing will result in a `ValueError` being raised.
    ///
    /// # Raises
    ///
    /// `ValueError`: If the device has already been freed.
    ///
    /// # Examples
    ///
    /// Construct a `RadioLink`, free it, then construct a `Motor` with the same Smart Port:
    ///
    /// ```python
    /// from venice import *
    ///
    /// link = RadioLink(1, "643A", LinkType.MANAGER)
    /// link.free()
    /// # `link` is now unusable, but port 1 can be used in another device, such as a `Motor`
    /// motor = Motor(1)
    /// ```
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }

    fn stream_read(self_in: Obj, buf: &mut [u8]) -> Result<usize, c_int> {
        self_in
            .as_obj::<RadioLinkObj>()
            .guard
            .borrow_mut()
            .read(buf)
            .map_err(err_to_code)
    }

    fn stream_write(self_in: Obj, buf: &[u8]) -> Result<usize, c_int> {
        self_in
            .as_obj::<RadioLinkObj>()
            .guard
            .borrow_mut()
            .write(buf)
            .map_err(err_to_code)
    }

    fn stream_ioctl(self_in: Obj, req: IoctlReq) -> Result<usize, c_int> {
        let this = self_in.as_obj::<RadioLinkObj>();
        let mut link = this.guard.borrow_mut();

        match req {
            IoctlReq::Poll(poll) => {
                let mut ret = Poll::empty();

                if poll.contains(Poll::RD) {
                    ret |= link
                        .unread_bytes()
                        .map(|b| if b > 0 { Poll::RD } else { Poll::empty() })
                        .unwrap_or(Poll::ERR);
                }

                if poll.contains(Poll::WR) {
                    ret |= link
                        .write_capacity()
                        .map(|b| if b > 0 { Poll::WR } else { Poll::empty() })
                        .unwrap_or(Poll::ERR);
                }

                Ok(ret.bits())
            }
            IoctlReq::Flush => {
                link.flush().map_err(err_to_code)?;
                Ok(0)
            }
            _ => Err(MP_EINVAL),
        }
    }

    #[stream]
    const STREAM: Stream = Stream {
        read: read_from_fn!(RadioLinkObj::stream_read),
        write: write_from_fn!(RadioLinkObj::stream_write),
        ioctl: ioctl_from_fn!(RadioLinkObj::stream_ioctl),
        is_text: 0, // uhh maybe
    };

    // TODO: find good docs for these stream methods

    #[constant(qstr!(read))]
    #[stub(sig = "(self, size: int = -1) -> bytes")]
    const READ: &FunVarBetween = &mp_stream_read_obj;
    #[constant(qstr!(read1))]
    #[stub(sig = "(self, size: int = -1) -> bytes")]
    const READ1: &FunVarBetween = &mp_stream_read1_obj;
    #[constant(qstr!(write))]
    #[stub(sig = "(self, buffer: bytes | bytearray | memoryview) -> int")]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;
    #[constant(qstr!(write1))]
    #[stub(sig = "(self, buffer: bytes | bytearray | memoryview) -> int")]
    const WRITE1: &Fun2 = &mp_stream_write1_obj;
    #[constant(qstr!(flush))]
    #[stub(sig = "(self) -> None")]
    const FLUSH: &Fun1 = &mp_stream_flush_obj;
    #[constant(qstr!(ioctl))]
    #[stub(sig = "(self, request: int, arg: int = 0) -> int")]
    const IOCTL: &FunVarBetween = &mp_stream_ioctl_obj;
}
