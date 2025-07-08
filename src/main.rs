//! Open source MicroPython port for VEX V5 robots.

#![no_std]
#![no_main]

mod micropython;
mod stubs;
mod vbt;

use core::{arch::naked_asm, fmt::Write};

use talc::{ErrOnOom, Span, Talc, Talck};

use crate::{micropython::MicroPython, vbt::MODULE_MAP};

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

// TODO: Synchronize properly once Python multitasking is added
struct Serial;

impl Write for Serial {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if unsafe { vex_sdk::vexSerialWriteChar(1, c as u8) } == -1 {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if unsafe { vex_sdk::vexSerialWriteBuffer(1, s.as_ptr(), s.len() as u32) } == -1 {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = write!(Serial, "panicked: {}", info.message());
    loop {}
}

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;

    /// Calls libc global constructors.
    ///
    /// # Safety
    ///
    /// Must be called once at the start
    fn __libc_init_array();

    /// Calls libc global destructors.
    ///
    /// # Safety
    ///
    /// Must be called once after
    fn __libc_fini_array();
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
///
/// # Safety
///
/// Must be called once after [`__libc_init_array`] has been called.
unsafe fn exit() -> ! {
    unsafe {
        __libc_fini_array();
        vex_sdk::vexSystemExitRequest();
    }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

fn main(mut mpy: MicroPython) {
    let entrypoint = MODULE_MAP
        .get(b"__init__".as_slice())
        .expect("__init__ module not found, try adding __init__.py to your project");

    mpy.exec_bytecode(*entrypoint);
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
        __libc_init_array();
    }

    unsafe extern "C" {
        static mut __heap_start: u8;
        static mut __heap_end: u8;
    }

    unsafe {
        ALLOCATOR
            .lock()
            .claim(Span::new(&raw mut __heap_start, &raw mut __heap_end))
            .expect("couldn't claim heap memory");
    }

    main(unsafe { MicroPython::new() });

    unsafe {
        exit();
    }
}
