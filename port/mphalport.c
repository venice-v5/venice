#include <stdint.h>

#include "py/mpconfig.h"

#include "sdk.h"

void mp_hal_stdout_tx_strn_cooked(const char* str, mp_uint_t len) {
	vexSerialWriteBuffer(1, (const uint8_t*)str, len);
}
