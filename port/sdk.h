#include <stddef.h>

static const size_t JUMP_TABLE_START = 0x037fc000;

#define MAP_JUMP_TABLE(ret_type, name, args, arg_names, offset) \
    static inline ret_type name args { \
        typedef ret_type (*fn_ptr) args; \
        fn_ptr fn = *(fn_ptr*)(JUMP_TABLE_START + (offset)); \
        return fn arg_names; \
    }

MAP_JUMP_TABLE(
	int32_t,
	vexSerialWriteBuffer,
	(uint32_t channel, const uint8_t* data, uint32_t len),
	(channel, data, len),
	0x89c
);

MAP_JUMP_TABLE(
	void,
	vexTasksRun,
	(void),
	(),
	0x05c
);

MAP_JUMP_TABLE(
	void,
	vexSystemExitRequest,
	(void),
	(),
	0x130
);
