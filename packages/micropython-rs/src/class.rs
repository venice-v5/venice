use crate::{
    map::Dict,
    obj::{Attr, BinaryOp, Iter, MakeNew, ObjType, Printer, Subscr, UnaryOp},
    stream::Stream,
};

pub trait Class {
    const PARENT: Option<&ObjType> = None;
    const LOCALS_DICT: Option<&Dict> = None;
    const MAKE_NEW: Option<MakeNew> = None;
    const ITER: Option<Iter> = None;
    const ATTR: Option<Attr> = None;
    const SUBSCR: Option<Subscr> = None;
    const STREAM: Option<&Stream> = None;
    const UNARY_OP: Option<UnaryOp> = None;
    const BINARY_OP: Option<BinaryOp> = None;
    const PRINTER: Option<Printer> = None;
}
