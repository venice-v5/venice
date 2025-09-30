__linked_file_length = 4M;
__scratch_heap_length = 128K;

OVERWRITE_SECTIONS {
    .heap (NOLOAD) : {
        __scratch_heap_start = .;
        . += __scratch_heap_length;
        __scratch_heap_end = .;
        __heap_start = .;
        . = __stack_bottom;
        __heap_end = .;
    }
}
