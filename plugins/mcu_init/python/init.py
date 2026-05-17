#!/usr/bin/env python3
"""
MCU Init Component — STM32 HAL Template Generator

Creates a ready-to-compile HAL template project under repos/<MCU_MODEL>/
with Makefile, linker script, and a minimal main.c entry point.

Supports fuzzy matching: when the user provides an imprecise model number,
candidates are listed and the user can select interactively.
"""

import argparse
import os
import sys
import re
import json
from datetime import datetime

# ── Component metadata ──
COMPONENT_ID = "mcu_init"
VERSION = "1.3.2"

# ── MCU Database ──
# Each entry maps the lowercased model to its hardware profile.
# flash_start is always 0x08000000 for STM32.
MCU_DATABASE = {
    # ── STM32F1 (Cortex-M3) ──
    "stm32f103c8":  {"cpu": "cortex-m3", "flash_kb":   64, "ram_kb":  20, "family": "F1", "march": "armv7-m"},
    "stm32f103cb":  {"cpu": "cortex-m3", "flash_kb":  128, "ram_kb":  20, "family": "F1", "march": "armv7-m"},
    "stm32f103rb":  {"cpu": "cortex-m3", "flash_kb":  128, "ram_kb":  20, "family": "F1", "march": "armv7-m"},
    "stm32f103rc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m"},
    "stm32f103rd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m"},
    "stm32f103re":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m"},
    "stm32f103rf":  {"cpu": "cortex-m3", "flash_kb":  768, "ram_kb":  96, "family": "F1", "march": "armv7-m"},
    "stm32f103rg":  {"cpu": "cortex-m3", "flash_kb": 1024, "ram_kb":  96, "family": "F1", "march": "armv7-m"},
    "stm32f103vc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m"},
    "stm32f103vd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m"},
    "stm32f103ve":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m"},
    "stm32f103zc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m"},
    "stm32f103zd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m"},
    "stm32f103ze":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m"},

    # ── STM32F4 (Cortex-M4, HW float) ──
    "stm32f405rg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f407vg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f407zg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f407ig":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f407zgt6":{"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f407vet6":{"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f411re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f411ce":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f412zg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 256, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f413zh":  {"cpu": "cortex-m4", "flash_kb": 1536, "ram_kb": 320, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f429zi":  {"cpu": "cortex-m4", "flash_kb": 2048, "ram_kb": 256, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f446re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f401re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb":  96, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32f401cc":  {"cpu": "cortex-m4", "flash_kb":  256, "ram_kb":  64, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},

    # ── STM32F7 (Cortex-M7) ──
    "stm32f746zg":  {"cpu": "cortex-m7", "flash_kb": 1024, "ram_kb": 320, "family": "F7", "march": "armv7e-m", "fpu": "fpv5-sp-d16"},
    "stm32f746ng":  {"cpu": "cortex-m7", "flash_kb": 1024, "ram_kb": 320, "family": "F7", "march": "armv7e-m", "fpu": "fpv5-sp-d16"},
    "stm32f767zi":  {"cpu": "cortex-m7", "flash_kb": 2048, "ram_kb": 512, "family": "F7", "march": "armv7e-m", "fpu": "fpv5-d16"},

    # ── STM32H7 (Cortex-M7, high performance) ──
    "stm32h743zi":  {"cpu": "cortex-m7", "flash_kb": 2048, "ram_kb": 1024, "family": "H7", "march": "armv7e-m", "fpu": "fpv5-d16"},
    "stm32h750vb":  {"cpu": "cortex-m7", "flash_kb":  128, "ram_kb": 1024, "family": "H7", "march": "armv7e-m", "fpu": "fpv5-d16"},

    # ── STM32G0 (Cortex-M0+) ──
    "stm32g030f6":  {"cpu": "cortex-m0plus", "flash_kb":  32, "ram_kb":   8, "family": "G0", "march": "armv6-m"},
    "stm32g031k8":  {"cpu": "cortex-m0plus", "flash_kb":  64, "ram_kb":   8, "family": "G0", "march": "armv6-m"},
    "stm32g071rb":  {"cpu": "cortex-m0plus", "flash_kb": 128, "ram_kb":  36, "family": "G0", "march": "armv6-m"},

    # ── STM32G4 (Cortex-M4, HW float) ──
    "stm32g431rb":  {"cpu": "cortex-m4", "flash_kb": 128, "ram_kb":  32, "family": "G4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32g474re":  {"cpu": "cortex-m4", "flash_kb": 512, "ram_kb": 128, "family": "G4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32g491re":  {"cpu": "cortex-m4", "flash_kb": 512, "ram_kb": 112, "family": "G4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},

    # ── STM32L0 (Cortex-M0+) ──
    "stm32l031k6":  {"cpu": "cortex-m0plus", "flash_kb":  32, "ram_kb":  8, "family": "L0", "march": "armv6-m"},
    "stm32l072cz":  {"cpu": "cortex-m0plus", "flash_kb": 192, "ram_kb": 20, "family": "L0", "march": "armv6-m"},

    # ── STM32L4 (Cortex-M4, HW float) ──
    "stm32l432kc":  {"cpu": "cortex-m4", "flash_kb": 256, "ram_kb":  64, "family": "L4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32l476rg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 128, "family": "L4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
    "stm32l496zg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 320, "family": "L4", "march": "armv7e-m", "fpu": "fpv4-sp-d16"},
}


# ── Helpers ──

def normalize(s: str) -> str:
    """Lowercase, strip, collapse whitespace."""
    return s.strip().lower()


def fuzzy_match(user_input: str) -> list:
    """
    Score all database entries against the user input.
    Returns a list of (model, score) sorted by score descending.
    Score rules:
      - exact match: 100
      - starts-with match: 80
      - substring match: 60
      - prefix without suffix match: 40
      - family match: 20
    """
    needle = normalize(user_input)
    results = []

    for model in MCU_DATABASE:
        score = 0
        if needle == model:
            score = 100
        elif model.startswith(needle):
            score = 80
        elif needle in model:
            score = 60
        else:
            # Try matching the core prefix, e.g. "f407" matches "stm32f407xx"
            core = re.sub(r'^stm32', '', model, count=1)
            if needle.lstrip('stm32') == core or needle in core:
                score = 50
            elif core.startswith(needle.lstrip('stm32')):
                score = 40
            # Family match
            if MCU_DATABASE[model].get("family", "").lower() in needle:
                score = max(score, 20)

        if score > 0:
            results.append((model, score))

    results.sort(key=lambda x: x[1], reverse=True)
    return results


def select_mcu(user_input: str) -> str | None:
    """
    Resolve user input to a canonical MCU model.
    Returns the model string on success, None on failure.
    """
    candidates = fuzzy_match(user_input)

    if not candidates:
        print(f"[!] 未找到匹配 '{user_input}' 的 MCU 型号")
        print("    使用 'mcu_init list' 查看所有支持的型号")
        return None

    # If there's an exact match (score 100), use it directly
    exact = [(m, s) for m, s in candidates if s == 100]
    if exact:
        model = exact[0][0]
        info = MCU_DATABASE[model]
        print(f"[✓] 精确匹配: {model.upper()} ({info['cpu']}, {info['flash_kb']}KB Flash, {info['ram_kb']}KB RAM)")
        return model

    if len(candidates) == 1 and candidates[0][1] >= 80:
        # High-confidence single match
        model = candidates[0][0]
        info = MCU_DATABASE[model]
        print(f"[✓] 自动匹配: {model.upper()} ({info['cpu']}, {info['flash_kb']}KB Flash, {info['ram_kb']}KB RAM)")
        return model

    # Multiple candidates or medium confidence — ask user
    print(f"[?] '{user_input}' 匹配到以下候选:")
    for i, (model, score) in enumerate(candidates[:10], 1):
        info = MCU_DATABASE[model]
        cpu = info['cpu']
        flash = info['flash_kb']
        ram = info['ram_kb']
        conf = f"({score}%)" if score < 100 else ""
        print(f"  {i:2d}. {model.upper():20s} {cpu:15s} {flash:4d}KB Flash  {ram:4d}KB RAM  {conf}")

    try:
        choice = input("\n选择序号 (1-{}) 或输入 'q' 取消: ".format(min(len(candidates), 10))).strip()
    except (EOFError, KeyboardInterrupt):
        print("\n[!] 已取消")
        return None

    if choice.lower() == 'q':
        print("[!] 已取消")
        return None

    try:
        idx = int(choice) - 1
        if 0 <= idx < min(len(candidates), 10):
            return candidates[idx][0]
    except ValueError:
        pass

    print(f"[!] 无效选择: '{choice}'")
    return None


# ── Template generators ──

def makefile_content(mcu: str, info: dict) -> str:
    """Generate Makefile for the given MCU."""
    upper = mcu.upper()
    cpu = info['cpu']
    march = info['march']
    fpu = info.get('fpu', None)
    flash_kb = info['flash_kb']
    ram_kb = info['ram_kb']

    fpu_flags = ""
    if fpu:
        fpu_flags = f"-mfpu={fpu} -mfloat-abi=hard"
    else:
        fpu_flags = "-mfloat-abi=soft"

    linker_script = f"{upper}_FLASH.ld"

    return f"""# Makefile for {upper} HAL Template
# Generated by rabber mcu_init v{VERSION}
# Target: {cpu} ({march}), {flash_kb}KB Flash, {ram_kb}KB RAM

# ── Toolchain ──
CC      = arm-none-eabi-gcc
OBJCOPY = arm-none-eabi-objcopy
OBJDUMP = arm-none-eabi-objdump
SIZE    = arm-none-eabi-size
GDB     = arm-none-eabi-gdb

# ── Target MCU ──
MCU      = {upper}
CPU      = {cpu}
FPU      = {fpu if fpu else 'none'}

# ── Directories ──
BUILD_DIR   = build
PROCESS_DIR = process
SRC_DIR     = src
INC_DIR     = include

# ── Source files ──
C_SRCS   = $(wildcard $(SRC_DIR)/*.c)
ASM_SRCS = $(wildcard $(SRC_DIR)/*.s)
OBJS     = $(C_SRCS:$(SRC_DIR)/%.c=$(PROCESS_DIR)/%.o) \\
           $(ASM_SRCS:$(SRC_DIR)/%.s=$(PROCESS_DIR)/%.o)

# ── Output ──
TARGET   = $(BUILD_DIR)/{upper.lower()}
ELF      = $(TARGET).elf
HEX      = $(TARGET).hex
BIN      = $(TARGET).bin

# ── Compiler flags ──
CFLAGS  = -mcpu={cpu}
CFLAGS += -march={march}
CFLAGS += -mthumb
CFLAGS += {fpu_flags}
CFLAGS += -std=gnu11
CFLAGS += -Wall -Wextra -Werror
CFLAGS += -Os -g3 -ggdb
CFLAGS += -ffunction-sections -fdata-sections
CFLAGS += -fno-common
CFLAGS += -D{chip_define(upper)}

# ── Includes ──
INCLUDES = -I$(INC_DIR)

# ── Linker flags ──
LDSCRIPT = {linker_script}
LDFLAGS  = -T$(LDSCRIPT)
LDFLAGS += -mcpu={cpu}
LDFLAGS += -mthumb
LDFLAGS += {fpu_flags}
LDFLAGS += -specs=nosys.specs
LDFLAGS += -specs=nano.specs
LDFLAGS += -Wl,--gc-sections
LDFLAGS += -Wl,-Map=$(PROCESS_DIR)/output.map
LDFLAGS += -Wl,--print-memory-usage

# ── Phony targets ──
.PHONY: all clean flash

# ── Default ──
all: $(HEX) $(BIN)
\t@echo ""
\t@echo "[✓] 构建完成:"
\t@$(SIZE) $(ELF)

# ── ELF ──
$(ELF): $(OBJS) $(LDSCRIPT)
\t@mkdir -p $(BUILD_DIR)
\t@echo "[LD]  $@"
\t$(CC) $(OBJS) $(LDFLAGS) -o $@

# ── HEX ──
$(HEX): $(ELF)
\t@echo "[HEX] $@"
\t$(OBJCOPY) -O ihex $< $@

# ── BIN ──
$(BIN): $(ELF)
\t@echo "[BIN] $@"
\t$(OBJCOPY) -O binary $< $@

# ── Object files ──
$(PROCESS_DIR)/%.o: $(SRC_DIR)/%.c
\t@mkdir -p $(PROCESS_DIR)
\t@echo "[CC]  $<"
\t$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(PROCESS_DIR)/%.o: $(SRC_DIR)/%.s
\t@mkdir -p $(PROCESS_DIR)
\t@echo "[AS]  $<"
\t$(CC) $(CFLAGS) -c $< -o $@

# ── Clean ──
clean:
\trm -rf $(BUILD_DIR) $(PROCESS_DIR)
\t@echo "[✓] 清理完成"

# ── Flash (via st-flash) ──
flash: $(BIN)
\tst-flash --reset write $< 0x08000000
\t@echo "[✓] 烧录完成"

# ── Debug ──
debug: $(ELF)
\t$(GDB) $<
"""


def chip_define(upper: str) -> str:
    """Convert stm32f407zgt6 to STM32F407ZGT6 (already upper)."""
    return upper


def linker_script_content(mcu: str, info: dict) -> str:
    """Generate a minimal GCC linker script for the MCU."""
    upper = mcu.upper()
    flash_kb = info['flash_kb']
    ram_kb = info['ram_kb']
    flash_len = flash_kb * 1024
    ram_len = ram_kb * 1024

    return f"""/* Linker script for {upper}
 * Flash: {flash_kb} KB, RAM: {ram_kb} KB
 * Generated by rabber mcu_init v{VERSION}
 */

MEMORY
{{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = {flash_len}
    RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = {ram_len}
}}

/* Top of stack = end of RAM */
_estack = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(Reset_Handler)

SECTIONS
{{
    /* ── Vector table ── */
    .isr_vector :
    {{
        . = ALIGN(4);
        KEEP(*(.isr_vector))
        . = ALIGN(4);
    }} > FLASH

    /* ── Program code ── */
    .text :
    {{
        . = ALIGN(4);
        *(.text)
        *(.text.*)
        *(.rodata)
        *(.rodata.*)
        *(.glue_7)
        *(.glue_7t)
        . = ALIGN(4);
        _etext = .;
    }} > FLASH

    /* ── Initialised data ── */
    _sidata = LOADADDR(.data);
    .data :
    {{
        . = ALIGN(4);
        _sdata = .;
        *(.data)
        *(.data.*)
        . = ALIGN(4);
        _edata = .;
    }} > RAM AT > FLASH

    /* ── Zero-initialised data ── */
    .bss :
    {{
        . = ALIGN(4);
        _sbss = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    }} > RAM

    /* ── Stack / heap (optional) ── */
    ._user_heap_stack :
    {{
        . = ALIGN(8);
        PROVIDE(end = .);
        PROVIDE(_end = .);
        . = . + 0x400;   /* 1 KB minimum stack */
        . = ALIGN(8);
    }} > RAM
}}
"""


def main_c_content(mcu: str, info: dict) -> str:
    """Generate a minimal main.c with a blinking LED template."""
    upper = mcu.upper()
    cpu = info['cpu']
    flash_kb = info['flash_kb']
    ram_kb = info['ram_kb']

    return f"""/**
 * {upper} HAL Template — main.c
 * Target: {cpu}, {flash_kb}KB Flash, {ram_kb}KB RAM
 * Generated by rabber mcu_init v{VERSION}
 *
 * Minimal example: initialises the system clock and blinks an LED on PC13
 * (the built-in LED on most STM32 boards).
 */

#include <stdint.h>

/* ── Memory-mapped peripheral base addresses ── */
#define PERIPH_BASE           0x40000000U
#define AHB1PERIPH_BASE       (PERIPH_BASE + 0x00020000U)
#define GPIOC_BASE            (AHB1PERIPH_BASE + 0x0800U)

/* ── RCC registers ── */
#define RCC_BASE              (AHB1PERIPH_BASE + 0x3800U)
#define RCC_AHB1ENR           (*(volatile uint32_t *)(RCC_BASE + 0x30U))
#define RCC_AHB1ENR_GPIOCEN   (1U << 2)

/* ── GPIO registers ── */
#define GPIO_MODER            (*(volatile uint32_t *)(GPIOC_BASE + 0x00U))
#define GPIO_OTYPER           (*(volatile uint32_t *)(GPIOC_BASE + 0x04U))
#define GPIO_OSPEEDR          (*(volatile uint32_t *)(GPIOC_BASE + 0x08U))
#define GPIO_ODR              (*(volatile uint32_t *)(GPIOC_BASE + 0x14U))
#define GPIO_BSRR             (*(volatile uint32_t *)(GPIOC_BASE + 0x18U))

/* ── SysTick ── */
#define STK_BASE              0xE000E010U
#define STK_CTRL              (*(volatile uint32_t *)(STK_BASE + 0x00U))
#define STK_LOAD              (*(volatile uint32_t *)(STK_BASE + 0x04U))
#define STK_VAL               (*(volatile uint32_t *)(STK_BASE + 0x08U))
#define STK_CTRL_ENABLE       (1U << 0)
#define STK_CTRL_TICKINT      (1U << 1)
#define STK_CTRL_CLKSOURCE    (1U << 2)

/* ── System clock ── */
/* Default HSI = 16 MHz, no PLL — suitable for blink demo. */
#define SYSTEM_CLOCK_HZ       16000000U

/* ── Global tick counter ── */
static volatile uint32_t sys_ticks = 0;

/* ── Linker-defined symbols ── */
extern uint32_t _estack;      /* top of stack (defined in linker script) */
extern uint32_t _sidata, _sdata, _edata, _sbss, _ebss;

/* ── Forward declarations ── */
void SystemInit(void);
void SysTick_Handler(void);
static void SysTick_Config(uint32_t ticks);
void delay_ms(uint32_t ms);
int main(void);

/* ── Reset handler ── */
__attribute__((naked, noreturn))
void Reset_Handler(void) {{
    uint32_t *src = &_sidata;
    uint32_t *dst = &_sdata;
    while (dst < &_edata) {{ *dst++ = *src++; }}
    dst = &_sbss;
    while (dst < &_ebss) {{ *dst++ = 0; }}

    SystemInit();
    main();
    while (1) {{ }}
}}

/* ── Default exception handlers ── */
__attribute__((weak, alias("Default_Handler")))
void NMI_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void HardFault_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void MemManage_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void BusFault_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void UsageFault_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void SVC_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void DebugMon_Handler(void);
__attribute__((weak, alias("Default_Handler")))
void PendSV_Handler(void);

void Default_Handler(void) {{
    while (1) {{ }}
}}

/* ── Vector table ── */
__attribute__((section(".isr_vector")))
void (*const g_pfnVectors[])(void) = {{
    (void (*)(void))(&_estack),          /* Initial stack pointer (top of RAM) */
    Reset_Handler,
    NMI_Handler,
    HardFault_Handler,
    MemManage_Handler,
    BusFault_Handler,
    UsageFault_Handler,
    0, 0, 0, 0,                        /* Reserved */
    SVC_Handler,
    DebugMon_Handler,
    0,                                  /* Reserved */
    PendSV_Handler,
    SysTick_Handler,
    /* ... additional IRQ handlers go here */
}};

/* ── System initialisation ── */
void SystemInit(void) {{
    /* Enable GPIOC clock */
    RCC_AHB1ENR |= RCC_AHB1ENR_GPIOCEN;

    /* Configure PC13 as push-pull output */
    GPIO_MODER   &= ~(3U << 26);   /* Clear mode bits */
    GPIO_MODER   |=  (1U << 26);   /* Output mode */
    GPIO_OTYPER  &= ~(1U << 13);   /* Push-pull */
    GPIO_OSPEEDR |=  (3U << 26);   /* High speed */

    /* Configure SysTick for 1 ms intervals */
    SysTick_Config(SYSTEM_CLOCK_HZ / 1000U);
}}

/* ── SysTick configuration ── */
static inline void SysTick_Config(uint32_t ticks) {{
    STK_LOAD = ticks - 1U;
    STK_VAL  = 0;
    STK_CTRL = STK_CTRL_CLKSOURCE | STK_CTRL_TICKINT | STK_CTRL_ENABLE;
}}

/* ── SysTick interrupt handler ── */
void SysTick_Handler(void) {{
    sys_ticks++;
}}

/* ── Millisecond delay ── */
void delay_ms(uint32_t ms) {{
    uint32_t start = sys_ticks;
    while ((sys_ticks - start) < ms) {{
        /* spin */
    }}
}}

/* ── Main ── */
int main(void) {{
    while (1) {{
        /* Toggle PC13 (on-board LED) */
        GPIO_BSRR = (1U << 13);   /* LED off (set bit) — assuming active-low */
        delay_ms(500);
        GPIO_BSRR = (1U << (13 + 16)); /* LED on (reset bit) */
        delay_ms(500);
    }}
    return 0;
}}
"""


# ── File system operations ──

def resolve_repos_root() -> str:
    """Find the repos/ directory: <project_root>/repos/"""
    # Try PROJECT_ROOT env var (set by Rust main)
    env_root = os.environ.get("PROJECT_ROOT", "")
    if env_root:
        repos = os.path.join(env_root, "repos")
        return repos

    # Try current working directory
    cwd = os.getcwd()
    # Walk up to find Cargo.toml
    cur = cwd
    for _ in range(5):
        if os.path.isfile(os.path.join(cur, "Cargo.toml")):
            return os.path.join(cur, "repos")
        parent = os.path.dirname(cur)
        if parent == cur:
            break
        cur = parent

    # Fallback: repos/ under cwd
    return os.path.join(cwd, "repos")


def create_template(mcu: str, info: dict, force: bool = False) -> str | None:
    """
    Create template files under repos/<MCU>/.
    Returns the target directory path on success, None on failure.
    """
    repos_root = resolve_repos_root()
    target_dir = os.path.join(repos_root, mcu.upper())

    if os.path.exists(target_dir):
        if not force:
            print(f"[!] 目录已存在: {target_dir}")
            print("    使用 --force 覆盖现有文件")
            return None
        print(f"[!] 覆盖现有目录: {target_dir}")

    # Create directory structure
    src_dir = os.path.join(target_dir, "src")
    inc_dir = os.path.join(target_dir, "include")
    for d in [target_dir, src_dir, inc_dir]:
        os.makedirs(d, exist_ok=True)

    # Write files
    files = {
        "Makefile": makefile_content(mcu, info),
        f"{mcu.upper()}_FLASH.ld": linker_script_content(mcu, info),
        os.path.join("src", "main.c"): main_c_content(mcu, info),
    }

    for rel_path, content in files.items():
        abs_path = os.path.join(target_dir, rel_path)
        with open(abs_path, "w") as f:
            f.write(content)
        print(f"  [✓] {rel_path}")

    print(f"\n[✓] 模板创建完成: {target_dir}")
    print(f"    进入目录后执行 'make' 编译")
    print(f"    或使用 'cc compile src/main.c' 通过插件编译器编译")

    return target_dir


# ── Action handlers ──

def action_init(args: argparse.Namespace) -> None:
    """Handle the 'init' action."""
    extra = getattr(args, 'extra', []) or []
    # Filter out leading '--' separator from plugin-loader / shell
    extra = [a for a in extra if a != '--']
    force = "--force" in extra
    mcu_input = " ".join([a for a in extra if not a.startswith("--")]) if extra else ""

    if not mcu_input:
        print("[!] 请指定 MCU 型号")
        print("用法: mcu_init init <MCU型号>")
        print("示例: mcu_init init stm32f407zgt6")
        sys.exit(1)

    model = select_mcu(mcu_input)
    if model is None:
        sys.exit(1)

    info = MCU_DATABASE[model]
    target = create_template(model, info, force=force)

    if target:
        # Signal to the shell that it should cd here.
        # The Rust shell reads this marker to automatically change directory.
        print(f"__RABBER_CD__:{target}")
    else:
        sys.exit(1)


def action_list(args: argparse.Namespace) -> None:
    """Handle the 'list' action."""
    extra = getattr(args, 'extra', []) or []
    extra = [a for a in extra if a != '--']
    filter_str = " ".join(extra).lower().strip() if extra else ""

    families = {}
    for model, info in sorted(MCU_DATABASE.items()):
        fam = info.get("family", "?")
        families.setdefault(fam, []).append((model, info))

    print(f"[MCU 数据库] {len(MCU_DATABASE)} 个型号\n")
    for fam in sorted(families.keys()):
        entries = families[fam]
        if filter_str and filter_str not in fam.lower():
            # Also check individual models
            filtered = [(m, i) for m, i in entries if filter_str in m]
            if not filtered:
                continue
            entries = filtered

        print(f"── STM32{fam} ──")
        for model, info in entries:
            cpu = info['cpu']
            flash = info['flash_kb']
            ram = info['ram_kb']
            fpu = info.get('fpu', '')
            fpu_str = f", FPU: {fpu}" if fpu else ""
            print(f"  {model.upper():20s} {cpu:15s} {flash:4d}KB Flash  {ram:4d}KB RAM{fpu_str}")
        print()


def action_info(args: argparse.Namespace) -> None:
    """Handle the 'info' action."""
    print(f"[MCU Init Component v{VERSION}]")
    print(f"  组件 ID: {COMPONENT_ID}")
    print(f"  描述: STM32 HAL 模板初始化器")
    print(f"  支持的型号: {len(MCU_DATABASE)} 个")
    print(f"  模板目录: repos/<MCU型号>/")
    print()
    print("  Families:")
    families = sorted(set(info.get("family", "?") for info in MCU_DATABASE.values()))
    print(f"    {', '.join(f'F{f}' for f in families)}")
    print()
    print("  支持的 Cortex 核心:")
    cores = sorted(set(info["cpu"] for info in MCU_DATABASE.values()))
    for c in cores:
        count = sum(1 for info in MCU_DATABASE.values() if info["cpu"] == c)
        print(f"    {c}: {count} 个型号")


# ── Main ──

def main():
    parser = argparse.ArgumentParser(
        description=f"MCU Init v{VERSION} — STM32 HAL Template Generator",
    )
    parser.add_argument("--action", required=True, help="Action: init, list, info")
    parser.add_argument("--file", help="Ignored (compatibility)")
    parser.add_argument("--address", help="Ignored (compatibility)")
    parser.add_argument("extra", nargs=argparse.REMAINDER, help="Extra arguments (use -- to separate)")

    args = parser.parse_args()

    actions = {
        "init": action_init,
        "list": action_list,
        "info": action_info,
    }

    handler = actions.get(args.action)
    if handler:
        handler(args)
    else:
        print(f"[!] 未知动作: {args.action}")
        print(f"    可用: {', '.join(actions.keys())}")
        sys.exit(1)


if __name__ == "__main__":
    main()
