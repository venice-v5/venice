//! Open source MicroPython port for VEX V5 robots.
#![feature(allocator_api)]

pub mod args;
pub mod devices;
pub mod fun;
pub mod obj;
pub mod qstrgen;
pub mod registry;

mod alloc;
mod exports;
mod module_map;
mod modvasyncio;
mod modvenice;
mod stubs;

use std::{
    io::{Write, stderr, stdout},
    panic::PanicHookInfo,
};

use micropython_rs::{
    init::{InitToken, init_mp},
    module::exec_module,
    nlr::push_nlr,
    qstr::Qstr,
};
use talc::Span;
use venice_program_table::Vpt;
use vex_sdk_jumptable as _;

use crate::{
    module_map::{MODULE_MAP, init_module_map},
    alloc::ALLOCATOR,
};

// TODO: pick another ID
const VENDOR_ID: u32 = 0x11235813;

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;

    static mut __fallback_heap_start: u8;
    static mut __fallback_heap_end: u8;

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
    // I/O relies on memory allocation. I/O functions called before the allocator is initialized
    // will fail.
    let token = unsafe {
        let token = init_mp(&raw mut __heap_start, &raw mut __heap_end).unwrap();

        {
            let fallback_heap_span =
                Span::new(&raw mut __fallback_heap_start, &raw mut __fallback_heap_end);
            ALLOCATOR
                .lock()
                .claim(fallback_heap_span)
                .unwrap();
        }
        std::panic::set_hook(Box::new(panic_hook));

        let vpt = Vpt::from_ptr(&raw const __linked_file_start, VENDOR_ID)
            .expect("invalid VPT was uploaded");
        init_module_map(vpt).unwrap();

        token
    };

    init_main(token);
    // for simulator
    stdout().flush().unwrap();
    stderr().flush().unwrap();
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
