//! Open source MicroPython port for VEX V5 robots.

#![no_std]
#![no_main]

mod micropython;
mod stubs;
mod vcbc;

use core::{arch::naked_asm, fmt::Write};

use crate::{
    micropython::MicroPython,
    vcbc::{BYTECODE_TABLE_MAGIC, BytecodeTable},
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

fn program(mut mpy: MicroPython) {
    let bytecode_table = BytecodeTable::get().unwrap_or_else(|magic| {
        panic!(
            "invalid bytecode table magic: should be 0x{BYTECODE_TABLE_MAGIC:08x}, got 0x{magic:08x}"
        )
    });

    let entrypoint = bytecode_table
        .module(0)
        .expect("no entrypoint in bytecode table");

    mpy.exec_bytecode(entrypoint);
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

    program(unsafe { MicroPython::new() });

    unsafe {
        exit();
    }
}
