#!/usr/bin/env python3
"""
CMSIS-DAP downloader component using pyOCD.

This module provides a plugin interface for CMSIS-DAP-based flashing,
info collection, reset, verify, and ELF debug stripping.
"""

import argparse
import hashlib
import os
import subprocess
import sys

COMPONENT_ID = "cmsis_dap"
VERSION = "1.0.0"
DESCRIPTION = "CMSIS-DAP downloader component using pyOCD for ARM Cortex-M flashing."


class CMSISDAPDownloader:
    def __init__(self):
        self.pyocd_cmd = self._find_command("pyocd")

    def _find_command(self, cmd):
        candidates = [cmd]
        if os.name == "nt":
            candidates.append(cmd + ".exe")
        for candidate in candidates:
            if self._command_exists(candidate):
                return candidate
        return candidates[0]

    def _command_exists(self, cmd):
        try:
            subprocess.run([cmd, "--version"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            return True
        except (FileNotFoundError, OSError):
            return False

    def run_command(self, args):
        try:
            result = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            if result.returncode != 0:
                print(result.stderr.strip(), file=sys.stderr)
            return result.returncode, result.stdout.strip()
        except FileNotFoundError:
            print(f"命令不存在: {args[0]}", file=sys.stderr)
            return 1, ""

    def probe(self):
        code, output = self.run_command([self.pyocd_cmd, "list"])
        if code != 0 or not output:
            return None
        info = {"probe": output}
        return info

    def get_info(self):
        code, output = self.run_command([self.pyocd_cmd, "info"])
        if code == 0 and output:
            return {"info": output}
        return self.probe()

    def strip(self, elf_file_path):
        if not os.path.exists(elf_file_path):
            print(f"ELF文件不存在: {elf_file_path}", file=sys.stderr)
            return None

        objcopy_cmd = self._find_command("arm-none-eabi-objcopy")
        if not self._command_exists(objcopy_cmd):
            objcopy_cmd = self._find_command("objcopy")
            if not self._command_exists(objcopy_cmd):
                print("找不到 objcopy 工具。请安装 binutils 或 arm-none-eabi-binutils。", file=sys.stderr)
                return None

        stripped_file = elf_file_path + ".stripped.elf"
        code, _ = self.run_command([objcopy_cmd, "--strip-debug", elf_file_path, stripped_file])
        if code != 0 or not os.path.exists(stripped_file):
            print("去除调试信息失败。", file=sys.stderr)
            return None

        print(f"已去除调试信息: {stripped_file}")
        return stripped_file

    def flash(self, file_path, start_address=0x08000000, verify=True):
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        actual_file = file_path
        cleanup_temp = False
        if file_path.lower().endswith(".elf"):
            stripped = self.strip(file_path)
            if stripped is None:
                return False
            actual_file = stripped
            cleanup_temp = True

        args = [self.pyocd_cmd, "flash", actual_file]
        if start_address != 0x08000000:
            args.extend(["--address", f"0x{start_address:08X}"])

        code, _ = self.run_command(args)
        if code != 0:
            if cleanup_temp:
                os.remove(actual_file)
            return False

        reset_ok = self.reset()
        if not reset_ok:
            if cleanup_temp:
                os.remove(actual_file)
            return False

        if verify:
            success = self.verify(actual_file, start_address)
        else:
            success = True

        if cleanup_temp:
            os.remove(actual_file)

        return success

    def verify(self, file_path, start_address=0x08000000):
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        file_hash = self._calculate_file_hash(file_path)
        print(f"固件文件哈希: {file_hash}")

        if self.probe() is None:
            print("验证失败: 无法探测 MCU", file=sys.stderr)
            return False

        print("验证成功: MCU 响应正常")
        return True

    def reset(self):
        code, _ = self.run_command([self.pyocd_cmd, "reset"])
        if code == 0:
            print("已成功复位 MCU。")
            return True
        print("复位 MCU 失败。", file=sys.stderr)
        return False

    def _calculate_file_hash(self, file_path):
        hash_sha256 = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_sha256.update(chunk)
        return hash_sha256.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--action", required=True, choices=["probe", "info", "flash", "reset", "verify", "strip"])
    parser.add_argument("--file", help="Firmware or ELF file path for flash/verify/strip action")
    parser.add_argument("--address", type=lambda x: int(x, 0), default=0x08000000, help="Start address for flash/verify (default: 0x08000000)")
    parser.add_argument("--no-verify", action="store_true", help="Skip verification after flash")
    args = parser.parse_args()

    downloader = CMSISDAPDownloader()

    if args.action == "probe":
        info = downloader.probe()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        return 1
    elif args.action == "info":
        info = downloader.get_info()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        return 1
    elif args.action == "flash":
        if not args.file:
            print("flash 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if downloader.flash(args.file, args.address, not args.no_verify) else 1
    elif args.action == "verify":
        if not args.file:
            print("verify 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if downloader.verify(args.file, args.address) else 1
    elif args.action == "reset":
        return 0 if downloader.reset() else 1
    elif args.action == "strip":
        if not args.file:
            print("strip 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if downloader.strip(args.file) else 1

    return 1


if __name__ == "__main__":
    sys.exit(main())
