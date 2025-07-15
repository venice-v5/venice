//! Open source MicroPython port for VEX V5 robots.

#![no_std]
#![no_main]

extern crate alloc;

mod libc;
mod micropython;
mod serial;
mod vbt;

use core::{
    arch::naked_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use talc::{ErrOnOom, Span, Talc, Talck};

use crate::{
    micropython::{MicroPython, Obj, Qstr},
    serial::{print, println},
    vbt::MODULE_MAP,
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
static ALLOCATOR: Talck<spin::Mutex<()>, ErrOnOom> = Talck::new(Talc::new(ErrOnOom));

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    static PANICKED: AtomicBool = AtomicBool::new(false);

    if PANICKED.load(Ordering::Acquire) {
        // SAFETY: The code leading up to and including the call to `__libc_init_array` is
        // infallible, so panicking before it is impossible.
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

    loop {}
}

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;

    static mut __heap_start: u8;
    static mut __heap_end: u8;
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
    const ENTRYPOINT_NAME: &[u8] = b"__init__".as_slice();

    let entrypoint = MODULE_MAP
        .get(ENTRYPOINT_NAME)
        .expect("__init__ module not found, try adding __init__.py to your project");

    let qstr = Obj::from_qstr(Qstr::from_bytes(ENTRYPOINT_NAME));
    mpy.exec_module(qstr, *entrypoint);
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

    main(unsafe { MicroPython::new() });
    exit();
}
