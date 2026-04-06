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

#[class(qstr!(RadioLink))]
pub struct RadioLinkObj {
    base: ObjBase,
    guard: SmartGuard<RadioLink>,
}

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

    #[constant]
    const MANAGER: &Self = &Self::new(LinkType::Manager);
    #[constant]
    const WORKER: &Self = &Self::new(LinkType::Worker);
}

#[class_methods]
impl RadioLinkObj {
    #[constant]
    const INTERNAL_BUFFER_SIZE: i32 = RadioLink::INTERNAL_BUFFER_SIZE as i32;

    #[make_new]
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

    #[method]
    fn is_linked(&self) -> bool {
        self.guard.borrow_mut().is_linked()
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }

    fn stream_read(self_in: Obj, buf: &mut [u8]) -> Result<usize, c_int> {
        self_in
            .try_as_obj::<RadioLinkObj>()
            .unwrap()
            .guard
            .borrow_mut()
            .read(buf)
            .map_err(err_to_code)
    }

    fn stream_write(self_in: Obj, buf: &[u8]) -> Result<usize, c_int> {
        self_in
            .try_as_obj::<RadioLinkObj>()
            .unwrap()
            .guard
            .borrow_mut()
            .write(buf)
            .map_err(err_to_code)
    }

    fn stream_ioctl(self_in: Obj, req: IoctlReq) -> Result<usize, c_int> {
        let this = self_in.try_as_obj::<RadioLinkObj>().unwrap();
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

    #[constant(qstr!(read))]
    const READ: &FunVarBetween = &mp_stream_read_obj;
    #[constant(qstr!(read1))]
    const READ1: &FunVarBetween = &mp_stream_read1_obj;
    #[constant(qstr!(write))]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;
    #[constant(qstr!(write1))]
    const WRITE1: &Fun2 = &mp_stream_write1_obj;
    #[constant(qstr!(flush))]
    const FLUSH: &Fun1 = &mp_stream_flush_obj;
    #[constant(qstr!(ioctl))]
    const IOCTL: &FunVarBetween = &mp_stream_ioctl_obj;
}
