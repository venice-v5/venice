//! Open source MicroPython port for VEX V5 robots.

#![no_std]
#![no_main]

extern crate alloc;

mod exports;
mod module_map;
mod qstrgen;
mod serial;
mod stubs;
mod vasyncio;

use alloc::string::String;
use core::{
    arch::naked_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use micropython_rs::{
    gc::LockedGc,
    init::{InitToken, init_mp},
    module::exec_module,
    nlr::push_nlr,
    qstr::Qstr,
};
use venice_program_table::Vpt;

use crate::{
    module_map::{add_vpt, lock_module_map},
    serial::{print, println},
};

/// Signature used by VEXos to verify the program and its properties.
#[used]
#[unsafe(link_section = ".code_signature")]
static CODE_SIG: (vex_sdk::vcodesig, [u32; 4]) = (
    vex_sdk::vcodesig {
        magic: vex_sdk::V5_SIG_MAGIC,
        r#type: vex_sdk::V5_SIG_TYPE_USER,
        owner: vex_sdk::V5_SIG_OWNER_PARTNER,
        // empty options
        options: 0,
    },
    [0; 4],
);

#[global_allocator]
static ALLOCATOR: LockedGc = LockedGc::new(None);

// TODO: pick another ID
const VENDOR_ID: u32 = 0x11235813;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    static PANICKED: AtomicBool = AtomicBool::new(false);

    if PANICKED.load(Ordering::Acquire) {
        exit();
    }

    PANICKED.store(true, Ordering::Release);

    println!("Venice panicked!");
    println!(
        "If you are seeing this message as a user, please file a bug report at https://github.com/venice-v5/venice\n"
    );

    if let Some(location) = info.location() {
        print!("[{}:{}]: ", location.file(), location.line());
    } else {
        print!("[no location available]: ");
    }

    println!("{}", info.message());

    exit();
}

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;

    static mut __heap_start: u8;
    static mut __heap_end: u8;

    static __linked_file_start: u8;
}

#[unsafe(link_section = ".boot")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _boot() -> ! {
    naked_asm!(
        "ldr sp, =__stack_top",
        "b {}",
        sym startup,
    );
}

/// Cleanly terminate program.
fn exit() -> ! {
    unsafe {
        vex_sdk::vexSystemExitRequest();
    }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

fn main(token: InitToken) {
    const VENICE_PACKAGE_NAME_PROGRAM: &[u8] = b"__venice__package_name__";

    let entrypoint_name = lock_module_map()
        .get(VENICE_PACKAGE_NAME_PROGRAM)
        .unwrap_or_else(|| {
            panic!(
                "malformed VPT: '{}' not present",
                str::from_utf8(VENICE_PACKAGE_NAME_PROGRAM).unwrap()
            )
        })
        .payload();

    let entrypoint_qstr = Qstr::from_bytes(entrypoint_name);

    let entrypoint = lock_module_map()
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

/// # Safety
///
/// Must be called once immediately after program boot.
unsafe fn startup() -> ! {
    let token = unsafe {
        let mut bss_ptr = &raw mut __bss_start;
        while bss_ptr < &raw mut __bss_end {
            bss_ptr.write_volatile(0);
            bss_ptr = bss_ptr.add(1);
        }

        let (token, gc) = init_mp(&raw mut __heap_start, &raw mut __heap_end).unwrap();
        *ALLOCATOR.lock() = Some(gc);

        let vpt = Vpt::from_ptr(&raw const __linked_file_start, VENDOR_ID)
            .expect("invalid VPT was uploaded");
        add_vpt(vpt);

        token
    };

    main(token);
    exit();
}
