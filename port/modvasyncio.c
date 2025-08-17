#include "py/runtime.h"

const mp_obj_module_t vasyncio_module = {
    .base = { &mp_type_module },
    .globals = 0,
};

MP_REGISTER_MODULE(MP_QSTR_vasyncio, vasyncio_module);
