#include "py/runtime.h"

extern const mp_obj_dict_t venice_globals;

const mp_obj_module_t venice_module = {
    .base = { &mp_type_module },
    .globals = &venice_globals,
};

MP_REGISTER_MODULE(MP_QSTR_venice, venice_module);
