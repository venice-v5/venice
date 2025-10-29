#include "py/runtime.h"

extern const mp_obj_dict_t vasyncio_globals;

const mp_obj_module_t vasyncio_module = {
    .base = { &mp_type_module },
    .globals = &vasyncio_globals,
};

MP_REGISTER_MODULE(MP_QSTR_vasyncio, vasyncio_module);
