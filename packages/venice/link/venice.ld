__linked_file_length = 4M;
__fallback_heap_length = 128K;

OVERWRITE_SECTIONS {
    .text ALIGN(4) : {
        *(.boot)
        *(.text .text.*)
    } > USER_RAM

    .heap (NOLOAD): {
        __fallback_heap_start = .;
        . += __fallback_heap_length;
        __fallback_heap_end = .;

        __heap_start = .;
        . = __stack_bottom;
        __heap_end = .;
    }
}
