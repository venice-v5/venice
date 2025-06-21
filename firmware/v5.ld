OUTPUT_FORMAT("elf32-littlearm")

/*
 * _boot is an assembly routine defined in src/lib.rs that sets
 * up the stack pointer before jumping to _start (Rust entrypoint).
 */
ENTRY(_boot)

MEMORY {
    USER_RAM (RWX) : ORIGIN = 0x03800000, LENGTH = 0x04000000
}

__code_signature_length = 0x20;
__stack_length = 0x00400000;

/* see https://github.com/llvm/llvm-project/blob/main/libunwind/src/AddressSpace.hpp#L78 */
__eh_frame_hdr_start = SIZEOF(.eh_frame_hdr) > 0 ? ADDR(.eh_frame_hdr) : 0;
__eh_frame_hdr_end = SIZEOF(.eh_frame_hdr) > 0 ? . : 0;

SECTIONS {
    /*
     * VEXos expects program binaries to have a 32-byte header called a "code signature",
     * at their start, which tells the OS that we are a valid program and configures some
     * miscellaneous startup behavior.
     */
    .code_signature : {
        KEEP(*(.code_signature))
		/* sets relative to ORIGIN(USER_RAM) */
		. = __code_signature_length;
    } > USER_RAM
	
	/* _boot routine (always runs first, must be at 0x03800020) */
	.boot : {
		*(.boot)
	} > USER_RAM

    /*
     * Executable program instructions.
     */
    .text : {
        *(.text .text.*)
    } > USER_RAM

    /*
     * Global/uninitialized/static/constant data sections.
     */
    .rodata : {
        *(.rodata .rodata.*)
    } > USER_RAM

    .data : {
        *(.data .data.*)
    } > USER_RAM

    /*
     * ARM Stack Unwinding Sections
     *
     * These sections are added by the compiler in some cases to facilitate stack unwinding.
     * __eh_frame_start and similar symbols are used by libunwind.
     */
    .eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr))
    } > USER_RAM

    .eh_frame : {
        __eh_frame_start = .;
       KEEP(*(.eh_frame))
        __eh_frame_end = .;
    } > USER_RAM

    .ARM.exidx : {
        __exidx_start = .;
        *(.ARM.exidx*)
        __exidx_end = .;
    } > USER_RAM

    .ARM.extab : {
        __extab_start = .;
        *(.ARM.extab*)
        __extab_end = .;
    } > USER_RAM
	
	/* -- End of loadable sections - anything beyond this point shouldn't go in the BIN. -- */

	/* Uninitialized data */
    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    } > USER_RAM

	/*
	 * Active memory locations for the stack/heap.
	 */
    .heap (NOLOAD) : ALIGN(4) {
        __heap_start = .;
        . = ORIGIN(USER_RAM) + LENGTH(USER_RAM) - __stack_length;
		__heap_end = .;
    } > USER_RAM

    .stack (NOLOAD) : ALIGN(8) {
        __stack_bottom = .;
        . += __stack_length;
        __stack_top = .;
    } > USER_RAM

    /*
     * `.ARM.attributes` contains arch metadata for compatibility purposes, but we
     * only target one hardware configuration, meaning it'd just take up space.
     */
    /DISCARD/ : {
        *(.ARM.attributes*)
    }
}
