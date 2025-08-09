MICROPYTHON = ../micropython

include $(MICROPYTHON)/py/mkenv.mk
include $(MICROPYTHON)/py/py.mk

CROSS_COMPILE = arm-none-eabi-
CFLAGS += -mcpu=cortex-a9 -mfpu=neon-fp16 -mfloat-abi=hard -mthumb -Os -g
CFLAGS += -I. -I$(MICROPYTHON) -I$(BUILD)

QSTR_DEFS += $(BUILD)/rust_qstrdefs.h

LIB = $(BUILD)/libmpyv5.a
OBJ += $(PY_CORE_O)

all: $(LIB)

$(LIB): $(OBJ)
	$(Q)$(AR) rcs $@ $^

include $(MICROPYTHON)/py/mkrules.mk
