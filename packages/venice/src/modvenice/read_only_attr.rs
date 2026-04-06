use argparse::error_msg;
use micropython_rs::{except::type_error, init::token, obj::ObjTrait};

pub fn read_only_attr<T: ObjTrait>() -> ! {
    let type_name = T::OBJ_TYPE.name().as_str();
    type_error(error_msg!(
        "attempt to modify attribute of '{type_name}'; '{type_name}' is read-only and its attributes cannot be written to"
    )).raise(token())
}
