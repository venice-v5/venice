#include <string.h>
#include <stdint.h>

#include "py/compile.h"
#include "py/gc.h"
#include "py/nlr.h"
#include "py/persistentcode.h"
#include "py/runtime.h"
#include "py/stackctrl.h"

#include "sdk.h"

// "VXV5"
static const uint32_t SIG_MAGIC = 0x35585658;

[[gnu::used]]
[[gnu::section(".code_signature")]]
uint32_t signature[8] = {
	SIG_MAGIC,
	// ProgramType::User
	0,
	// ProgramOwner::Partner
	2,
	// ProgramFlags::empty()
	0,
};

// prints the Fibonacci sequence from F(0) to F(19)
static const uint16_t PROGRAM[] = {
	0x064d, 0x1f00, 0x0106, 0x660c, 0x6269, 0x702e, 0x0079, 0x810f,
	0x0629, 0x6966, 0x0062, 0x6e02, 0x8100, 0x0577, 0x660c, 0x6269,
	0x7b28, 0x297d, 0x3d20, 0x7b20, 0x007d, 0x6c82, 0x0830, 0x8401,
	0x2608, 0x0032, 0x0316, 0x4280, 0x5758, 0x0416, 0x0511, 0x0023,
	0x0214, 0x0411, 0x0311, 0x0411, 0x0134, 0x0236, 0x0134, 0x8159,
	0x57e5, 0xd794, 0x2343, 0x5159, 0x0163, 0x4882, 0x0e21, 0x0403,
	0x2520, 0x2522, 0xb042, 0xd980, 0x4244, 0x6380, 0x81b0, 0x44d9,
	0x8142, 0x1263, 0xb003, 0xf381, 0x0134, 0x0312, 0x82b0, 0x34f3,
	0xf201, 0x5163, 0x0063,
};

extern volatile uint32_t __bss_start;
extern volatile uint32_t __bss_end;

extern uint8_t __stack_top;

extern uint8_t __heap_start;
extern uint8_t __heap_end;

void exec_program() {
	nlr_buf_t nlr;
	if (nlr_push(&nlr) == 0) {
		mp_module_context_t* ctxt = m_new_obj(mp_module_context_t);
		ctxt->module.globals = mp_globals_get();
		mp_compiled_module_t cm;
		cm.context = ctxt;
		mp_raw_code_load_mem((const uint8_t*)PROGRAM, sizeof(PROGRAM), &cm);
		mp_obj_t fn = mp_make_function_from_proto_fun(cm.rc, ctxt, MP_OBJ_NULL);
		mp_call_function_0(fn);
		nlr_pop();
	} else {
		mp_obj_print_exception(&mp_plat_print, (mp_obj_t)nlr.ret_val);
		while (1) {}
	}
}

void _start(void) {
	volatile uint32_t* bss_ptr = &__bss_start;
	while (bss_ptr < &__bss_end) {
		*bss_ptr = 0;
		bss_ptr++;
	}

	mp_stack_set_top(&__stack_top);
	gc_init(&__heap_start, &__heap_end);
	mp_init();

	exec_program();

	vexSystemExitRequest();

	while (1) {
		vexTasksRun();
	}
}

// shouldn't happen if our code was written correctly
void nlr_jump_fail(void* val) {
	while (1) {}
}

uint32_t store_gc_regs(uint32_t regs[10]);

void gc_collect(void) {
	gc_collect_start();
	uint32_t regs[10];
	uint32_t sp = store_gc_regs(regs);
	gc_collect_root((void**)sp, ((uint32_t)MP_STATE_THREAD(stack_top) - sp) / sizeof(uint32_t));
	gc_collect_end();
}
