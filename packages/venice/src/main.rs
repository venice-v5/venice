//! Open source MicroPython port for VEX V5 robots.

mod exports;
mod module_map;
mod obj;
mod qstrgen;
mod stubs;
mod vasyncio;

use micropython_rs::{
    gc::LockedGc,
    init::{InitToken, init_mp},
    module::exec_module,
    nlr::push_nlr,
    qstr::Qstr,
};
use venice_program_table::Vpt;

use crate::module_map::{MODULE_MAP, init_module_map};

#[global_allocator]
static ALLOCATOR: LockedGc = LockedGc::new(None);

// TODO: pick another ID
const VENDOR_ID: u32 = 0x11235813;

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;

    static mut __heap_start: u8;
    static mut __heap_end: u8;

    static __linked_file_start: u8;
}

fn init_main(token: InitToken) {
    const VENICE_PACKAGE_NAME_PROGRAM: &[u8] = b"__venice__package_name__";

    let entrypoint_name = MODULE_MAP
        .get()
        .unwrap()
        .get(VENICE_PACKAGE_NAME_PROGRAM)
        .unwrap_or_else(|| {
            panic!(
                "malformed VPT: '{}' not present",
                str::from_utf8(VENICE_PACKAGE_NAME_PROGRAM).unwrap()
            )
        })
        .payload();

    let entrypoint_qstr = Qstr::from_bytes(entrypoint_name);

    let entrypoint = MODULE_MAP
        .get()
        .unwrap()
        .get(entrypoint_qstr.bytes())
        .unwrap_or_else(|| {
            panic!(
                "malformed VPT: package '{}' not present",
                String::from_utf8_lossy(entrypoint_name)
            )
        })
        .payload();

    push_nlr(token, || exec_module(token, entrypoint_qstr, entrypoint));
}

fn main() {
    let token = unsafe {
        let (token, gc) = init_mp(&raw mut __heap_start, &raw mut __heap_end).unwrap();
        *ALLOCATOR.lock() = Some(gc);

        let vpt = Vpt::from_ptr(&raw const __linked_file_start, VENDOR_ID)
            .expect("invalid VPT was uploaded");
        init_module_map(vpt).unwrap();

        token
    };

    init_main(token);
}
