use micropython_rs::{const_dict, const_map, map::ConstDict};

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static vasyncio_globals: ConstDict = const_dict![];
