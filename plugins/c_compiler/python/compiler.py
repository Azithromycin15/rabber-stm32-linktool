#!/usr/bin/env python3
"""
C Compiler Component

This Python module implements a C source compiler component for the rabber-stm32-linktool.
It compiles C source files into ELF, HEX, and BIN formats, with build outputs saved to
the current working directory's build/ folder and intermediate files to process/.

Supports:
- arm-none-eabi-gcc toolchain for STM32/ARM MCU targets
- System gcc as fallback for local testing
- Logging to the project's logs/ directory
"""

import argparse
import subprocess
import sys
import os
import shutil
from datetime import datetime

# Component constants
COMPONENT_ID = "c_compiler"
VERSION = "1.0.0"
DESCRIPTION = "C source compiler for MCU firmware (ELF/HEX/BIN output)"

# Default flash start address for STM32
DEFAULT_FLASH_ADDRESS = "0x08000000"

# Known MCU targets and their GCC flags
MCU_TARGETS = {
    "stm32f103c8": {
        "cpu": "cortex-m3",
        "march": "armv7-m",
        "mthumb": True,
        "float": "soft",
    },
    "stm32f103cb": {
        "cpu": "cortex-m3",
        "march": "armv7-m",
        "mthumb": True,
        "float": "soft",
    },
    "stm32f407vg": {
        "cpu": "cortex-m4",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv4-sp-d16",
    },
    "stm32f411re": {
        "cpu": "cortex-m4",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv4-sp-d16",
    },
    "stm32f746zg": {
        "cpu": "cortex-m7",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv5-sp-d16",
    },
    "stm32h743zi": {
        "cpu": "cortex-m7",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv5-d16",
    },
    "stm32g431rb": {
        "cpu": "cortex-m4",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv4-sp-d16",
    },
    "stm32l476rg": {
        "cpu": "cortex-m4",
        "march": "armv7e-m",
        "mthumb": True,
        "float": "hard",
        "fpu": "fpv4-sp-d16",
    },
}


class Logger:
    """Simple file logger that writes to the project's logs/ directory."""

    def __init__(self):
        self.log_path = self._resolve_log_path()

    def _resolve_log_path(self):
        """Resolve the log file path from environment or create a new one."""
        # Check environment variable set by Rust main program
        env_log = os.environ.get("RABBER_LOG_FILE", "")
        if env_log and os.path.isdir(os.path.dirname(env_log)):
            return env_log

        # Fallback: find logs/ directory relative to cwd or project root
        cwd = os.getcwd()
        candidates = [
            os.path.join(cwd, "logs"),
            os.path.join(cwd, "..", "logs"),
        ]
        for cand in candidates:
            if os.path.isdir(cand):
                timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
                return os.path.join(cand, f"compile-{timestamp}.log")

        # Last resort: create logs/ in cwd
        logs_dir = os.path.join(cwd, "logs")
        os.makedirs(logs_dir, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        return os.path.join(logs_dir, f"compile-{timestamp}.log")

    def log(self, level, message):
        """Write a log entry."""
        try:
            parent = os.path.dirname(self.log_path)
            if parent:
                os.makedirs(parent, exist_ok=True)
            with open(self.log_path, "a") as f:
                timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                f.write(f"[{timestamp}] [{level}] {message}\n")
        except Exception:
            pass  # Silently fail if we can't write logs

    def info(self, message):
        """Log an INFO level message."""
        self.log("INFO", message)
        print(f"[INFO] {message}")

    def warn(self, message):
        """Log a WARN level message."""
        self.log("WARN", message)
        print(f"[WARN] {message}", file=sys.stderr)

    def error(self, message):
        """Log an ERROR level message."""
        self.log("ERROR", message)
        print(f"[ERROR] {message}", file=sys.stderr)


class CCompiler:
    """
    C Compiler class for compiling C source to ELF/HEX/BIN formats.

    Uses arm-none-eabi-gcc toolchain when targeting an MCU,
    or system gcc for native compilation.
    """

    def __init__(self, cwd=None):
        self.cwd = cwd or os.getcwd()
        self.logger = Logger()

        # Detect toolchains
        self.arm_gcc = self._find_tool("arm-none-eabi-gcc")
        self.arm_objcopy = self._find_tool("arm-none-eabi-objcopy")
        self.arm_size = self._find_tool("arm-none-eabi-size")
        self.system_gcc = self._find_tool("gcc")
        self.system_objcopy = self._find_tool("objcopy")

        # Output directories (relative to cwd)
        self.build_dir = os.path.join(self.cwd, "build")
        self.process_dir = os.path.join(self.cwd, "process")

    def _find_tool(self, name):
        """Find a tool executable path."""
        path = shutil.which(name)
        if path:
            return path
        return None

    def _ensure_dirs(self):
        """Ensure build/ and process/ directories exist."""
        os.makedirs(self.build_dir, exist_ok=True)
        os.makedirs(self.process_dir, exist_ok=True)
        self.logger.info(f"构建目录: {self.build_dir}")
        self.logger.info(f"过程目录: {self.process_dir}")

    def info(self):
        """Print toolchain information."""
        print(f"[C Compiler Component v{VERSION}]")
        print(f"  工作目录: {self.cwd}")
        print(f"  构建输出目录: {self.build_dir}")
        print(f"  过程文件目录: {self.process_dir}")
        print()
        print("检测到的工具链:")
        print(f"  arm-none-eabi-gcc:      {'已安装' if self.arm_gcc else '未安装'} ({self.arm_gcc or 'N/A'})")
        print(f"  arm-none-eabi-objcopy:  {'已安装' if self.arm_objcopy else '未安装'} ({self.arm_objcopy or 'N/A'})")
        print(f"  arm-none-eabi-size:     {'已安装' if self.arm_size else '未安装'} ({self.arm_size or 'N/A'})")
        print(f"  system gcc:             {'已安装' if self.system_gcc else '未安装'} ({self.system_gcc or 'N/A'})")
        print(f"  system objcopy:         {'已安装' if self.system_objcopy else '未安装'} ({self.system_objcopy or 'N/A'})")
        print()
        if self.arm_gcc:
            print("已知 MCU 目标:")
            for mcu, config in sorted(MCU_TARGETS.items()):
                print(f"  {mcu}: {config['cpu']} ({config['march']})")
        self.logger.info("显示编译器工具链信息")

    def clean(self):
        """Clean build and process directories."""
        self.logger.info("开始清理构建目录...")
        for d in [self.build_dir, self.process_dir]:
            if os.path.isdir(d):
                try:
                    shutil.rmtree(d)
                    self.logger.info(f"已删除目录: {d}")
                except Exception as e:
                    self.logger.error(f"删除目录失败 {d}: {e}")
            else:
                self.logger.info(f"目录不存在，跳过: {d}")
        print("清理完成。")

    def compile(self, source_file, mcu=None, flash_address=DEFAULT_FLASH_ADDRESS):
        """
        Compile a C source file to ELF, HEX, and BIN formats.

        Args:
            source_file (str): Path to the C source file
            mcu (str): Target MCU identifier (e.g., stm32f103c8)
            flash_address (str): Flash start address in hex

        Returns:
            bool: True if compilation succeeded
        """
        if not os.path.isfile(source_file):
            self.logger.error(f"源文件不存在: {source_file}")
            return False

        self._ensure_dirs()

        # Determine the base name for output files
        base_name = os.path.splitext(os.path.basename(source_file))[0]
        self.logger.info(f"开始编译: {source_file}")
        self.logger.info(f"输出基名: {base_name}")

        # Select toolchain
        if mcu:
            return self._compile_for_mcu(source_file, base_name, mcu, flash_address)
        else:
            return self._compile_native(source_file, base_name, flash_address)

    def _compile_for_mcu(self, source_file, base_name, mcu, flash_address):
        """Compile for a specific MCU target using arm-none-eabi-gcc."""
        if mcu.lower() not in MCU_TARGETS:
            self.logger.warn(f"未知 MCU 目标 '{mcu}'，使用默认 cortex-m3 编译选项")

        mcu_config = MCU_TARGETS.get(mcu.lower(), {"cpu": "cortex-m3", "march": "armv7-m", "mthumb": True, "float": "soft"})

        if not self.arm_gcc:
            self.logger.error("未找到 arm-none-eabi-gcc 工具链。请安装 gcc-arm-none-eabi。")
            return False

        self.logger.info(f"目标 MCU: {mcu}")
        self.logger.info(f"CPU: {mcu_config['cpu']}, arch: {mcu_config['march']}")

        # Step 1: Compile C -> Object file (process/)
        obj_file = os.path.join(self.process_dir, f"{base_name}.o")
        self.logger.info(f"步骤 1/4: 编译 C 文件 -> {obj_file}")

        compile_args = [
            self.arm_gcc,
            "-c",
            "-mcpu=" + mcu_config["cpu"],
            "-march=" + mcu_config["march"],
            "-mthumb" if mcu_config.get("mthumb") else "",
            "-mfloat-abi=" + mcu_config.get("float", "soft"),
        ]

        # Add FPU flag if specified
        if "fpu" in mcu_config:
            compile_args.append("-mfpu=" + mcu_config["fpu"])

        compile_args.extend([
            "-DSTM32",
            f"-D{mcu.upper() if mcu else 'STM32F103C8'}",
            "-O2",
            "-g",
            "-Wall",
            "-Wextra",
            "-ffunction-sections",
            "-fdata-sections",
            "-fno-common",
            source_file,
            "-o", obj_file,
        ])

        # Filter out empty strings
        compile_args = [a for a in compile_args if a]

        self.logger.info(f"执行: {' '.join(compile_args)}")
        result = subprocess.run(compile_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error("编译失败:")
            for line in result.stderr.strip().split("\n"):
                if line.strip():
                    self.logger.error(f"  {line}")
            return False
        self.logger.info("C 文件编译成功")
        if result.stderr.strip():
            for line in result.stderr.strip().split("\n"):
                if "warning" in line.lower():
                    self.logger.warn(f"  {line}")

        # Step 2: Link object -> ELF (build/)
        elf_file = os.path.join(self.build_dir, f"{base_name}.elf")
        self.logger.info(f"步骤 2/4: 链接 -> {elf_file}")

        # For bare-metal, we don't use standard lib. Use -nostdlib and provide linker script.
        # Since we can't assume a linker script is available, use --specs=nosys.specs for basic STM32 linking
        link_args = [
            self.arm_gcc,
            "-mcpu=" + mcu_config["cpu"],
            "-march=" + mcu_config["march"],
            "-mthumb" if mcu_config.get("mthumb") else "",
            "-mfloat-abi=" + mcu_config.get("float", "soft"),
            "-specs=nosys.specs",
            "-specs=nano.specs",
            "-Ttext=" + flash_address,
            "-Wl,--gc-sections",
            "-Wl,-Map=" + os.path.join(self.process_dir, f"{base_name}.map"),
            obj_file,
            "-o", elf_file,
        ]
        if "fpu" in mcu_config:
            link_args.insert(4, "-mfpu=" + mcu_config["fpu"])

        link_args = [a for a in link_args if a]

        self.logger.info(f"执行: {' '.join(link_args)}")
        result = subprocess.run(link_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error("链接失败:")
            for line in result.stderr.strip().split("\n"):
                if line.strip():
                    self.logger.error(f"  {line}")
            return False
        self.logger.info("链接成功")

        # Show size info
        if self.arm_size and os.path.isfile(elf_file):
            size_result = subprocess.run([self.arm_size, elf_file], capture_output=True, text=True)
            if size_result.returncode == 0:
                for line in size_result.stdout.strip().split("\n"):
                    print(f"  {line}")
                self.logger.info(f"固件大小:\n{size_result.stdout.strip()}")

        # Step 3: ELF -> HEX (build/)
        return self._convert_elf_to_formats(elf_file, base_name, flash_address)

    def _compile_native(self, source_file, base_name, flash_address):
        """Compile natively using system gcc (for testing)."""
        if not self.system_gcc:
            self.logger.error("未找到系统 gcc。")
            return False

        self.logger.info("使用系统 gcc 进行本地编译（非嵌入式目标）")

        # Step 1: Compile C -> Object file (process/)
        obj_file = os.path.join(self.process_dir, f"{base_name}.o")
        self.logger.info(f"步骤 1/3: 编译 C 文件 -> {obj_file}")

        compile_args = [
            self.system_gcc,
            "-c",
            "-O2",
            "-g",
            "-Wall",
            source_file,
            "-o", obj_file,
        ]
        self.logger.info(f"执行: {' '.join(compile_args)}")
        result = subprocess.run(compile_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error("编译失败:")
            for line in result.stderr.strip().split("\n"):
                if line.strip():
                    self.logger.error(f"  {line}")
            return False
        self.logger.info("C 文件编译成功")

        # Step 2: Link object -> ELF (build/)
        elf_file = os.path.join(self.build_dir, f"{base_name}.elf")
        self.logger.info(f"步骤 2/3: 链接 -> {elf_file}")

        link_args = [
            self.system_gcc,
            "-Wl,-Map=" + os.path.join(self.process_dir, f"{base_name}.map"),
            obj_file,
            "-o", elf_file,
        ]
        self.logger.info(f"执行: {' '.join(link_args)}")
        result = subprocess.run(link_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error("链接失败:")
            for line in result.stderr.strip().split("\n"):
                if line.strip():
                    self.logger.error(f"  {line}")
            return False
        self.logger.info("链接成功")

        # Step 3: Convert to HEX/BIN
        return self._convert_elf_to_formats(elf_file, base_name, flash_address, use_arm=False)

    def _convert_elf_to_formats(self, elf_file, base_name, flash_address, use_arm=True):
        """Convert ELF to HEX and BIN formats."""
        objcopy = self.arm_objcopy if use_arm else self.system_objcopy
        if not objcopy:
            self.logger.warn("未找到 objcopy，跳过 HEX/BIN 生成")
            print(f"\n输出文件: {elf_file}")
            return True

        hex_file = os.path.join(self.build_dir, f"{base_name}.hex")
        bin_file = os.path.join(self.build_dir, f"{base_name}.bin")

        # Step 3: ELF -> HEX
        self.logger.info(f"步骤 3/4: 生成 HEX -> {hex_file}")
        hex_args = [objcopy, "-O", "ihex", elf_file, hex_file]
        self.logger.info(f"执行: {' '.join(hex_args)}")
        result = subprocess.run(hex_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error(f"HEX 生成失败: {result.stderr.strip()}")
            return False
        self.logger.info("HEX 文件生成成功")

        # Step 4: ELF -> BIN
        self.logger.info(f"步骤 4/4: 生成 BIN -> {bin_file}")
        bin_args = [objcopy, "-O", "binary", elf_file, bin_file]
        self.logger.info(f"执行: {' '.join(bin_args)}")
        result = subprocess.run(bin_args, capture_output=True, text=True)
        if result.returncode != 0:
            self.logger.error(f"BIN 生成失败: {result.stderr.strip()}")
            return False
        self.logger.info("BIN 文件生成成功")

        # Print summary
        print()
        print("=" * 60)
        print("  编译完成！")
        print("=" * 60)
        for f in [elf_file, hex_file, bin_file]:
            if os.path.isfile(f):
                size = os.path.getsize(f)
                print(f"  {os.path.basename(f):<20} {size:>8d} bytes  ({f})")
        print(f"  过程文件目录: {self.process_dir}")
        print("=" * 60)

        self.logger.info(f"编译完成: {base_name}.elf, {base_name}.hex, {base_name}.bin")
        return True


def main():
    """Main entry point for the compiler component."""
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--action", required=True, choices=["compile", "info", "clean"])
    parser.add_argument("--file", help="C source file path for compile action")
    parser.add_argument("--mcu", default=None, help="Target MCU (e.g., stm32f103c8)")
    parser.add_argument("--address", type=str, default=DEFAULT_FLASH_ADDRESS,
                        help=f"Flash start address (default: {DEFAULT_FLASH_ADDRESS})")
    args = parser.parse_args()

    compiler = CCompiler()

    if args.action == "info":
        compiler.info()
        return 0

    elif args.action == "clean":
        compiler.clean()
        return 0

    elif args.action == "compile":
        if not args.file:
            print("compile 操作需要指定 --file <source.c>", file=sys.stderr)
            return 2

        success = compiler.compile(args.file, mcu=args.mcu, flash_address=args.address)
        return 0 if success else 1

    return 1


if __name__ == "__main__":
    sys.exit(main())
