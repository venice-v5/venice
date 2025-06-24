OUTPUT_FORMAT("elf32-littlearm")

/*
 * _boot is an assembly routine defined in src/lib.rs that sets
 * up the stack pointer before jumping to _start (Rust entrypoint).
 */
ENTRY(_boot)

__runtime_ram_start = 0x03800000;
__runtime_ram_length = 0x04000000;
__runtime_ram_end = __runtime_ram_start + __runtime_ram_length;

__ram_end = 0x08000000;
__bytecode_ram_start = __runtime_ram_end;
__bytecode_ram_length = __ram_end - __bytecode_ram_start;

__code_signature_length = 0x20;
__stack_length = 0x00400000;
__heap_end = __runtime_ram_end - __stack_length;

/* see https://github.com/llvm/llvm-project/blob/main/libunwind/src/AddressSpace.hpp#L78 */
__eh_frame_hdr_start = SIZEOF(.eh_frame_hdr) > 0 ? ADDR(.eh_frame_hdr) : 0;
__eh_frame_hdr_end = SIZEOF(.eh_frame_hdr) > 0 ? . : 0;

MEMORY {
    RUNTIME_RAM (RWX) : ORIGIN = __runtime_ram_start, LENGTH = __runtime_ram_length
	BYTECODE_RAM (R) : ORIGIN = __bytecode_ram_start, LENGTH = __bytecode_ram_length
}

SECTIONS {
    /*
     * VEXos expects program binaries to have a 32-byte header called a "code signature",
     * at their start, which tells the OS that we are a valid program and configures some
     * miscellaneous startup behavior.
     */
    .code_signature : {
        KEEP(*(.code_signature))
		/* sets relative to ORIGIN(RUNTIME_RAM) */
		. = __runtime_ram_start + __code_signature_length;
    } > RUNTIME_RAM

    /*
     * Executable program instructions.
     */
    .text : {
		/* _boot routine (always runs first, must be at 0x03800020) */
		*(.boot)
        *(.text .text.*)
    } > RUNTIME_RAM

    /*
     * Global/static/constant data sections.
     */
    .rodata : {
        *(.rodata .rodata.*)
    } > RUNTIME_RAM

    .data : {
        *(.data .data.*)
    } > RUNTIME_RAM

    /*
     * ARM Stack Unwinding Sections
     *
     * These sections are added by the compiler in some cases to facilitate stack unwinding.
     * __eh_frame_start and similar symbols are used by libunwind.
     */
    .eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr))
    } > RUNTIME_RAM

    .eh_frame : {
        __eh_frame_start = .;
       KEEP(*(.eh_frame))
        __eh_frame_end = .;
    } > RUNTIME_RAM

    .ARM.exidx : {
        __exidx_start = .;
        *(.ARM.exidx*)
        __exidx_end = .;
    } > RUNTIME_RAM

    .ARM.extab : {
        __extab_start = .;
        *(.ARM.extab*)
        __extab_end = .;
    } > RUNTIME_RAM
	
	/* -- End of loadable sections - anything beyond this point shouldn't go in the BIN. -- */

	/* Uninitialized data */
    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    } > RUNTIME_RAM

	/*
	 * Active memory locations for the stack/heap.
	 */
    .heap (NOLOAD) : ALIGN(4) {
        __heap_start = .;
		. = __heap_end;
    } > RUNTIME_RAM

    .stack (NOLOAD) : ALIGN(8) {
        __stack_bottom = .;
        . += __stack_length;
        __stack_top = .;
    } > RUNTIME_RAM

	.bytecode (NOLOAD) : ALIGN(4) {
		. = __bytecode_ram_start;
		. += __bytecode_ram_length;
	} > BYTECODE_RAM

    /*
     * `.ARM.attributes` contains arch metadata for compatibility purposes, but we
     * only target one hardware configuration, meaning it'd just take up space.
     */
    /DISCARD/ : {
        *(.ARM.attributes*)
    }

    /*
     * These sections are useful for debugging but shouldn't end up in the final
     * binary, so they are only stored in the ELF and not loaded into memory.
     */

    /* ELF metadata sections */
    .symtab   0 : { *(.symtab) }
    .strtab   0 : { *(.strtab) }
    .shstrtab 0 : { *(.shstrtab) }
    .debug    0 : { *(.debug*) }
    .comment  0 : { *(.comment) }
}
