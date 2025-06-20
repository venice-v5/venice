#include <stdint.h>

// disables all features
#define MICROPY_CONFIG_ROM_LEVEL (MICROPY_CONFIG_ROM_LEVEL_MINIMUM)

#define MICROPY_ENABLE_GC (1)
// loading bytecode
#define MICROPY_PERSISTENT_CODE_LOAD (1)

typedef int32_t mp_int_t; // must be pointer size
typedef uint32_t mp_uint_t; // must be pointer size
typedef long mp_off_t;

// We need to provide a declaration/definition of alloca()
#include <alloca.h>

#define MICROPY_HW_BOARD_NAME "VEX V5 Brain"
#define MICROPY_HW_MCU_NAME "Cortex-A9"
