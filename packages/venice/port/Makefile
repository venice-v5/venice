MICROPYTHON = ../micropython

include $(MICROPYTHON)/py/mkenv.mk
include $(MICROPYTHON)/py/py.mk

CROSS_COMPILE = arm-none-eabi-
CFLAGS += -mcpu=cortex-a9 -mfpu=neon-fp16 -mfloat-abi=hard -mthumb -Os
CFLAGS += -I. -I$(MICROPYTHON) -I$(BUILD)

SRC_C = modvenice.c
SRC_QSTR += modvenice.c

QSTR_DEFS += venice_qstrdefs.h

LIB = $(BUILD)/libmpyv5.a
OBJ += $(PY_CORE_O) $(addprefix $(BUILD)/, $(SRC_C:.c=.o))

all: $(LIB)

$(LIB): $(OBJ)
	$(Q)$(AR) rcs $@ $^

include $(MICROPYTHON)/py/mkrules.mk
