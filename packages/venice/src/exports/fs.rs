use micropython_rs::obj::Obj;

#[unsafe(no_mangle)]
static mut mp_builtin_open_obj: Obj = Obj::NONE;
