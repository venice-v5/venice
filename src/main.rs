//! Open source MicroPython port for VEX V5 robots.

#![no_std]
#![no_main]

extern crate alloc;

mod exports;
mod serial;
mod stubs;

use core::{
    arch::naked_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use micropython_rs::{MicroPython, obj::Obj, qstr::Qstr};
use talc::{ErrOnOom, Span, Talc, Talck};
use venice_program_table::Vpt;

use crate::serial::{print, println};

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
static ALLOCATOR: Talck<spin::Mutex<()>, ErrOnOom> = Talck::new(Talc::new(ErrOnOom));

// TODO: pick another ID
pub const VENDOR_ID: u32 = 0x11235813;

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

    static mut __python_heap_start: u8;
    static mut __python_heap_end: u8;

    static __bytecode_ram_start: u8;
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

fn main(mut mpy: MicroPython) {
    let entrypoint_name = Qstr::from_bytes(b"__init__");

    let entrypoint = match mpy.module_map().get(entrypoint_name.bytes()) {
        Some(bc) => bc,
        None => {
            println!("__init__ module not found, try adding __init__.py to your project");
            exit();
        }
    };

    let qstr_obj = Obj::from_qstr(entrypoint_name);
    mpy.exec_module(qstr_obj, *entrypoint);
}

/// # Safety
///
/// Must be called once immediately after program boot.
unsafe fn startup() -> ! {
    let mut bss_ptr = &raw mut __bss_start;
    while bss_ptr < &raw mut __bss_end {
        unsafe {
            bss_ptr.write_volatile(0);
            bss_ptr = bss_ptr.add(1);
        }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .claim(Span::new(&raw mut __heap_start, &raw mut __heap_end))
            .expect("couldn't claim heap memory");
    }

    let mut mp =
        unsafe { MicroPython::new(&raw mut __python_heap_start, &raw mut __python_heap_end) }
            .unwrap();

    mp.add_vpt(
        unsafe { Vpt::from_ptr(&raw const __bytecode_ram_start, VENDOR_ID) }
            .expect("invalid VPT was uploaded"),
    );

    main(mp);
    exit();
}
