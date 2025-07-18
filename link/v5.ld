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
__heap_length = 128K;
__python_heap_end = __runtime_ram_end - __stack_length;

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
	.heap (NOLOAD): ALIGN(4) {
		__heap_start = .;
		. += __heap_length;
		__heap_end = .;
	}

	.python_heap (NOLOAD) : ALIGN(4) {
		__python_heap_start = .;
		. = __python_heap_end;
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

	/* Stabs debugging sections.  */
	.stab		  0 : { *(.stab) }
	.stabstr	   0 : { *(.stabstr) }
	.stab.excl	 0 : { *(.stab.excl) }
	.stab.exclstr  0 : { *(.stab.exclstr) }
	.stab.index	0 : { *(.stab.index) }
	.stab.indexstr 0 : { *(.stab.indexstr) }
	.comment	   0 : { *(.comment) }
	.gnu.build.attributes : { *(.gnu.build.attributes .gnu.build.attributes.*) }
	/* DWARF debug sections.
	   Symbols in the DWARF debugging sections are relative to the beginning
	   of the section so we begin them at 0.  */
	/* DWARF 1.  */
	.debug		  0 : { *(.debug) }
	.line		   0 : { *(.line) }
	/* GNU DWARF 1 extensions.  */
	.debug_srcinfo  0 : { *(.debug_srcinfo) }
	.debug_sfnames  0 : { *(.debug_sfnames) }
	/* DWARF 1.1 and DWARF 2.  */
	.debug_aranges  0 : { *(.debug_aranges) }
	.debug_pubnames 0 : { *(.debug_pubnames) }
	/* DWARF 2.  */
	.debug_info	 0 : { *(.debug_info .gnu.linkonce.wi.*) }
	.debug_abbrev   0 : { *(.debug_abbrev) }
	.debug_line	 0 : { *(.debug_line .debug_line.* .debug_line_end) }
	.debug_frame	0 : { *(.debug_frame) }
	.debug_str	  0 : { *(.debug_str) }
	.debug_loc	  0 : { *(.debug_loc) }
	.debug_macinfo  0 : { *(.debug_macinfo) }
	/* SGI/MIPS DWARF 2 extensions.  */
	.debug_weaknames 0 : { *(.debug_weaknames) }
	.debug_funcnames 0 : { *(.debug_funcnames) }
	.debug_typenames 0 : { *(.debug_typenames) }
	.debug_varnames  0 : { *(.debug_varnames) }
	/* DWARF 3.  */
	.debug_pubtypes 0 : { *(.debug_pubtypes) }
	.debug_ranges   0 : { *(.debug_ranges) }
	/* DWARF 5.  */
	.debug_addr	 0 : { *(.debug_addr) }
	.debug_line_str 0 : { *(.debug_line_str) }
	.debug_loclists 0 : { *(.debug_loclists) }
	.debug_macro	0 : { *(.debug_macro) }
	.debug_names	0 : { *(.debug_names) }
	.debug_rnglists 0 : { *(.debug_rnglists) }
	.debug_str_offsets 0 : { *(.debug_str_offsets) }
	.debug_sup	  0 : { *(.debug_sup) }
	.gnu.attributes 0 : { KEEP (*(.gnu.attributes)) }
}
