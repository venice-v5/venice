use crate::{
    map::Dict,
    obj::{Attr, BinaryOpFn, Iter, MakeNew, ObjType, Subscr, UnaryOpFn},
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
    const UNARY_OP: Option<UnaryOpFn> = None;
    const BINARY_OP: Option<BinaryOpFn> = None;
}
