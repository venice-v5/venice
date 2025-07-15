#include <stdint.h>

#define MICROPY_CONFIG_ROM_LEVEL (MICROPY_CONFIG_ROM_LEVEL_BASIC_FEATURES)

#define MICROPY_ENABLE_COMPILER (0)
#define MICROPY_ENABLE_GC (1)
#define MICROPY_PERSISTENT_CODE_LOAD (1)

#define MICROPY_ENABLE_EXTERNAL_IMPORT (0)
#define MICROPY_PY_IO (0)

#define MICROPY_ERROR_REPORTING MICROPY_ERROR_REPORTING_DETAILED
#define MICROPY_WARNINGS (1)

#define MICROPY_LONGINT_IMPL MICROPY_LONGINT_IMPL_MPZ
#define MICROPY_FLOAT_IMPL MICROPY_FLOAT_IMPL_DOUBLE

typedef int32_t mp_int_t; // must be pointer size
typedef uint32_t mp_uint_t; // must be pointer size
typedef long mp_off_t;

// We need to provide a declaration/definition of alloca()
#include <alloca.h>

#define MICROPY_HW_BOARD_NAME "VEX V5 Brain"
#define MICROPY_HW_MCU_NAME "Cortex-A9"

#define mp_builtin___import__ venice_import
