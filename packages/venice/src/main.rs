//! Open source MicroPython port for VEX V5 robots.

pub mod args;
pub mod fun;
pub mod module_map;
pub mod obj;
pub mod qstrgen;

mod devices;
mod exports;
mod modvenice;
mod registry;
mod stubs;

use std::{
    io::{Write, stderr, stdout},
    panic::PanicHookInfo,
};

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
    let entrypoint_qstr = Qstr::from_bytes(b"main");

    let entrypoint = MODULE_MAP
        .get()
        .unwrap()
        .get(entrypoint_qstr.bytes())
        .unwrap_or_else(|| {
            panic!(
                "malformed VPT: package 'main' not present",
            )
        })
        .payload();

    push_nlr(token, || exec_module(token, entrypoint_qstr, entrypoint));
}

fn main() {
    // I/O relies on memory allocation. I/O functions called before the allocator is initialized
    // will fail.
    let token = unsafe {
        let (token, gc) = init_mp(&raw mut __heap_start, &raw mut __heap_end).unwrap();
        *ALLOCATOR.lock() = Some(gc);
        std::panic::set_hook(Box::new(panic_hook));

        let vpt = Vpt::from_ptr(&raw const __linked_file_start, VENDOR_ID)
            .expect("invalid VPT was uploaded");
        init_module_map(vpt).unwrap();

        token
    };

    init_main(token);
}

fn panic_hook(info: &PanicHookInfo) {
    // TODO: display on brain screen
    eprintln!("Venice panicked!");
    eprintln!(
        "If you see this message as a user, please file a bug report at https://github.com/venice-v5/venice/issues\n"
    );

    eprintln!("{info}");
    // for simulator
    stdout().flush().unwrap();
    stderr().flush().unwrap();
}
