MICROPYTHON = ../micropython
FIRMWARE = ../firmware
BUILD = ../build

include $(MICROPYTHON)/py/mkenv.mk
include $(MICROPYTHON)/py/py.mk

MFLAGS = -mcpu=cortex-a9 -mfpu=neon-fp16 -mfloat-abi=hard -mthumb -Os -g
CPPFLAGS = -I. -I$(MICROPYTHON) -I$(BUILD)
GCCFLAGS = -fdiagnostics-color -funwind-tables
WARNFLAGS = -Wall
C_STANDARD = gnu23

CROSS_COMPILE = arm-none-eabi-
CFLAGS = $(CPPFLAGS) $(MFLAGS) $(GCCFLAGS) $(WARNFLAGS) -std=$(C_STANDARD)
# $(BUILD) refers to ./build by default and contains qstr.generated.h, so it must be added as an
# include path
LDFLAGS = $(MFLAGS) $(GCCFLAGS) -nostdlib -nostartfiles -Wl,-T$(FIRMWARE)/v5.ld
LIBRARIES = -lgcc -L$(FIRMWARE) -lc -lm

NAME = venice
ELF = $(BUILD)/$(NAME).elf
BIN = $(BUILD)/$(NAME).bin

SRC_C = $(wildcard *.c)
SRC_S = $(wildcard *.S)
OBJ = $(PY_CORE_O) $(addprefix $(BUILD)/, $(SRC_C:.c=.o) $(SRC_S:.S=.o))

all: $(BIN)

$(BIN): $(ELF)
	$(Q)$(OBJCOPY) --output-target binary $(ELF) $(BIN)

$(ELF): $(OBJ)
	$(Q)$(CC) $(LDFLAGS) -o $@ $^ $(LIBRARIES)

include $(MICROPYTHON)/py/mkrules.mk
