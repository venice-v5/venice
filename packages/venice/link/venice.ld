__linked_file_length = 4M;
OVERWRITE_SECTIONS {
    .text ALIGN(4) : {
        *(.boot)
        *(.text .text.*)
    } > USER_RAM
}
