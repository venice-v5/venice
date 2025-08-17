use micropython_rs::{const_dict, const_map, map::ConstDict, obj::Obj};

use crate::qstrgen::qstr;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static vasyncio_globals: ConstDict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio))
];
