use std::{
    cell::RefCell,
    ffi::c_int,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

use argparse::{ArgParser, Args, DefaultParser, KeywordError, ParseError, StrParser, type_name};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    errno::{MP_EBADF, MP_EINVAL, MP_EIO},
    except::{Message, raise_os_error, type_error},
    fun::{Fun1, Fun2, FunVarBetween, FunVarKw},
    init::token,
    ioctl_from_fn,
    map::Map,
    obj::{Obj, ObjBase, ObjTrait},
    read_from_fn,
    stream::{
        IoctlReq, SEEK_CUR, SEEK_END, SEEK_SET, Stream, mp_stream_close_obj, mp_stream_flush_obj,
        mp_stream_ioctl_obj, mp_stream_read_obj, mp_stream_read1_obj, mp_stream_readinto_obj,
        mp_stream_seek_obj, mp_stream_tell_obj, mp_stream_write_obj, mp_stream_write1_obj,
    },
    write_from_fn,
};

use crate::{fun::fun_var_kw, modvenice::Exception};

#[class(qstr!(File))]
#[repr(C)]
pub struct FileObj {
    base: ObjBase,
    file: RefCell<Option<File>>,
}

#[class_methods]
impl FileObj {
    fn stream_read(self_in: Obj, buf: &mut [u8]) -> Result<usize, c_int> {
        let this = self_in.try_as_obj::<FileObj>().unwrap();
        this.file
            .borrow_mut()
            .as_mut()
            .ok_or(MP_EBADF)?
            .read(buf)
            .map_err(io_to_errno)
    }

    fn stream_write(self_in: Obj, buf: &[u8]) -> Result<usize, c_int> {
        let this = self_in.try_as_obj::<FileObj>().unwrap();
        this.file
            .borrow_mut()
            .as_mut()
            .ok_or(MP_EBADF)?
            .write(buf)
            .map_err(io_to_errno)
    }

    fn stream_ioctl(self_in: Obj, req: IoctlReq) -> Result<usize, c_int> {
        let this = self_in.try_as_obj::<FileObj>().unwrap();
        let mut file_opt = this.file.borrow_mut();
        let file = file_opt.as_mut().ok_or(MP_EBADF)?;

        match req {
            IoctlReq::Seek(seek) => {
                let seek_from = match seek.whence {
                    SEEK_SET => SeekFrom::Start(seek.offset as u64),
                    SEEK_CUR => SeekFrom::Current(seek.offset as i64),
                    SEEK_END => SeekFrom::End(seek.offset as i64),
                    _ => panic!("MicroPython lied..."),
                };

                file.seek(seek_from).map_err(io_to_errno)?;
            }
            IoctlReq::Flush => file.sync_all().map_err(io_to_errno)?,
            IoctlReq::Close => {
                // sync_all before closing to catch errors that would otherwise be silenced by the
                // destructor
                file.sync_all().map_err(io_to_errno)?;
                *file_opt = None;
            }
            _ => return Err(MP_EINVAL),
        }

        Ok(0)
    }

    #[stream]
    const STREAM: Stream = Stream {
        read: read_from_fn!(FileObj::stream_read),
        write: write_from_fn!(FileObj::stream_write),
        ioctl: ioctl_from_fn!(FileObj::stream_ioctl),
        is_text: 0,
    };

    #[constant(qstr!(read))]
    const READ: &FunVarBetween = &mp_stream_read_obj;

    #[constant(qstr!(read1))]
    const READ1: &FunVarBetween = &mp_stream_read1_obj;

    #[constant(qstr!(readinto))]
    const READINTO: &FunVarBetween = &mp_stream_readinto_obj;

    #[constant(qstr!(write))]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;

    #[constant(qstr!(write1))]
    const WRITE1: &Fun2 = &mp_stream_write1_obj;

    #[constant(qstr!(close))]
    const CLOSE: &Fun1 = &mp_stream_close_obj;

    #[constant(qstr!(seek))]
    const SEEK: &FunVarBetween = &mp_stream_seek_obj;

    #[constant(qstr!(tell))]
    const TELL: &Fun1 = &mp_stream_tell_obj;

    #[constant(qstr!(flush))]
    const FLUSH: &Fun1 = &mp_stream_flush_obj;

    #[constant(qstr!(ioctl))]
    const IOCTL: &FunVarBetween = &mp_stream_ioctl_obj;
}

struct Mode(OpenOptions);

impl Default for Mode {
    fn default() -> Self {
        let mut opts = OpenOptions::new();
        opts.read(true);
        Self(opts)
    }
}

#[derive(Default)]
struct ModeParser;

impl<'a> ArgParser<'a> for ModeParser {
    type Output = Mode;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        let mode_str = StrParser.parse(obj)?;

        let mut opts = OpenOptions::new();

        let mut has_base_mode = false;
        let mut plus_modifier = false;

        fn multiple_base_nodes(_: &str) -> Message {
            c"invalid mode: multiple base nodes".into()
        }

        for c in mode_str.chars() {
            match c {
                // Base modes
                'r' => {
                    if has_base_mode {
                        return Err(argparse::ParseError::ValueError {
                            mk_msg: Box::from(multiple_base_nodes),
                        });
                    }
                    opts.read(true);
                    has_base_mode = true;
                }
                'w' => {
                    if has_base_mode {
                        return Err(argparse::ParseError::ValueError {
                            mk_msg: Box::from(multiple_base_nodes),
                        });
                    }
                    opts.write(true).create(true).truncate(true);
                    has_base_mode = true;
                }
                'a' => {
                    if has_base_mode {
                        return Err(argparse::ParseError::ValueError {
                            mk_msg: Box::from(multiple_base_nodes),
                        });
                    }
                    opts.write(true).create(true).append(true);
                    has_base_mode = true;
                }
                'x' => {
                    if has_base_mode {
                        return Err(argparse::ParseError::ValueError {
                            mk_msg: Box::from(multiple_base_nodes),
                        });
                    }
                    opts.write(true).create_new(true);
                    has_base_mode = true;
                }

                // Modifiers
                '+' => {
                    plus_modifier = true;
                }
                'b' | 't' => {
                    // Valid Python mode characters, but standard Rust fs ignores
                    // text vs binary distinction. We accept them and do nothing.
                }
                _ => {
                    return Err(ParseError::ValueError {
                        mk_msg: Box::from(|_: &str| c"invalid mode character".into()),
                    });
                }
            }
        }

        if !has_base_mode {
            return Err(ParseError::ValueError {
                mk_msg: Box::from(|_: &str| {
                    c"mode must specify exactly one of 'r', 'w', 'a', or 'x'".into()
                }),
            });
        }

        // Apply the '+' modifier which enables the missing read/write flag
        if plus_modifier {
            opts.read(true).write(true);
        }

        Ok(Mode(opts))
    }
}

impl DefaultParser<'_> for Mode {
    type Parser = ModeParser;
}

fn io_to_errno(e: std::io::Error) -> c_int {
    e.raw_os_error().unwrap_or(MP_EIO)
}

fn open_inner(pos_args: &[Obj], kw_map: &Map) -> Result<FileObj, Exception> {
    let mut reader = Args::new(pos_args.len(), 0, pos_args).reader();
    reader.assert_npos(1, 2);

    let path = reader.next_positional::<&str>()?;

    let mode_kw_obj = kw_map.get(Obj::from_qstr(qstr!(mode)));
    let mode_obj = if let Ok(m) = reader.next_positional::<Obj>() {
        if mode_kw_obj.is_some() {
            return Err(
                type_error(c"argument 'mode' passed as both positional and keyword").into(),
            );
        }
        Some(m)
    } else {
        mode_kw_obj
    };

    let mode = mode_obj
        .as_ref()
        .map(|m| ModeParser.parse(m))
        .unwrap_or(Ok(Mode::default()))
        .map_err(|e| match e {
            ParseError::TypeError { expected } => KeywordError::TypeError {
                kw: "mode",
                expected,
                found: type_name(mode_obj.as_ref().unwrap()),
            },
            ParseError::ValueError { mk_msg } => KeywordError::ValueError {
                msg: mk_msg("argument 'mode'"),
            },
        })?;

    let file = mode
        .0
        .open(path)
        .unwrap_or_else(|e| raise_os_error(token(), io_to_errno(e)));

    Ok(FileObj {
        base: ObjBase::new(FileObj::OBJ_TYPE),
        file: RefCell::new(Some(file)),
    })
}

fn open(pos_args: &[Obj], kw_map: &Map) -> Obj {
    open_inner(pos_args, kw_map).into()
}

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
// cant use #[fun] cause it only generates an Obj :(
static mut mp_builtin_open_obj: FunVarKw = fun_var_kw!(open, 1);
