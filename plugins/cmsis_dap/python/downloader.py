#!/usr/bin/env python3
"""
CMSIS-DAP downloader component — OpenOCD first, pyOCD fallback.

When `openocd.cfg` is present in the current working directory, all operations
(probe, info, flash, reset, verify) use OpenOCD via the CMSIS-DAP interface.
If no `openocd.cfg` is found, operations fall back to pyOCD.
"""

import argparse
import hashlib
import os
import subprocess
import sys

COMPONENT_ID = "cmsis_dap"
VERSION = "1.3.3-Hotfix"
DESCRIPTION = "CMSIS-DAP downloader component — OpenOCD first, pyOCD fallback."


class CMSISDAPDownloader:
    def __init__(self):
        self.pyocd_cmd = self._find_command("pyocd")
        self.openocd_cmd = self._find_command("openocd")
        self._ocd_cfg = self._find_openocd_cfg()

    # ── helpers ──

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
            subprocess.run([cmd, "--version"],
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            return True
        except (FileNotFoundError, OSError):
            return False

    def run_command(self, args):
        try:
            result = subprocess.run(args, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, text=True)
            if result.returncode != 0:
                print(result.stderr.strip(), file=sys.stderr)
            return result.returncode, result.stdout.strip()
        except FileNotFoundError:
            print(f"命令不存在: {args[0]}", file=sys.stderr)
            return 1, ""

    def _find_openocd_cfg(self):
        """Return path to openocd.cfg if it exists in cwd, else None."""
        cfg = os.path.join(os.getcwd(), "openocd.cfg")
        return cfg if os.path.isfile(cfg) else None

    @property
    def use_openocd(self):
        return self._ocd_cfg is not None and self._command_exists(self.openocd_cmd)

    # ── probe ──

    def probe(self):
        if self.use_openocd:
            return self._probe_openocd()
        return self._probe_pyocd()

    def _probe_openocd(self):
        code, output = self.run_command([
            self.openocd_cmd, "-f", self._ocd_cfg,
            "-c", "init; targets; shutdown"
        ])
        if code == 0 and output:
            return {"probe": output}
        return None

    def _probe_pyocd(self):
        code, output = self.run_command([self.pyocd_cmd, "list"])
        if code != 0 or not output:
            return None
        return {"probe": output}

    # ── info ──

    def get_info(self):
        if self.use_openocd:
            return self._info_openocd()
        return self._info_pyocd()

    def _info_openocd(self):
        code, output = self.run_command([
            self.openocd_cmd, "-f", self._ocd_cfg,
            "-c", "init; flash banks; shutdown"
        ])
        if code == 0 and output:
            return {"info": output}
        return self.probe()

    def _info_pyocd(self):
        code, output = self.run_command([self.pyocd_cmd, "info"])
        if code == 0 and output:
            return {"info": output}
        return self.probe()

    # ── strip ──

    def strip(self, elf_file_path):
        if not os.path.exists(elf_file_path):
            print(f"ELF文件不存在: {elf_file_path}", file=sys.stderr)
            return None

        objcopy_cmd = self._find_command("arm-none-eabi-objcopy")
        if not self._command_exists(objcopy_cmd):
            objcopy_cmd = self._find_command("objcopy")
            if not self._command_exists(objcopy_cmd):
                print("找不到 objcopy 工具。请安装 binutils 或 arm-none-eabi-binutils。",
                      file=sys.stderr)
                return None

        stripped_file = elf_file_path + ".stripped.elf"
        code, _ = self.run_command(
            [objcopy_cmd, "--strip-debug", elf_file_path, stripped_file])
        if code != 0 or not os.path.exists(stripped_file):
            print("去除调试信息失败。", file=sys.stderr)
            return None

        print(f"已去除调试信息: {stripped_file}")
        return stripped_file

    # ── flash ──

    def flash(self, file_path, start_address=0x08000000, verify=True):
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        if self.use_openocd:
            return self._flash_openocd(file_path, start_address, verify)

        return self._flash_pyocd(file_path, start_address, verify)

    def _flash_openocd(self, file_path, start_address, verify):
        addr_str = f"0x{start_address:08X}" if start_address != 0x08000000 else "0x08000000"
        verify_cmd = " verify" if verify else ""
        print(f"[*] OpenOCD (CMSIS-DAP) → {file_path}")
        code, _ = self.run_command([
            self.openocd_cmd, "-f", self._ocd_cfg,
            "-c", f"program {file_path} {addr_str}{verify_cmd} reset exit"
        ])
        if code != 0:
            print("OpenOCD 刷写失败", file=sys.stderr)
            return False
        print("OpenOCD 刷写完成")
        return True

    def _flash_pyocd(self, file_path, start_address, verify):
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

    # ── verify ──

    def verify(self, file_path, start_address=0x08000000):
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        file_hash = self._calculate_file_hash(file_path)
        print(f"固件文件哈希: {file_hash}")

        if self.use_openocd:
            return self._verify_openocd(file_path, start_address)

        if self.probe() is None:
            print("验证失败: 无法探测 MCU", file=sys.stderr)
            return False
        print("验证成功: MCU 响应正常")
        return True

    def _verify_openocd(self, file_path, start_address):
        addr_str = f"0x{start_address:08X}" if start_address != 0x08000000 else "0x08000000"
        code, _ = self.run_command([
            self.openocd_cmd, "-f", self._ocd_cfg,
            "-c", f"init; verify_image {file_path} {addr_str}; shutdown"
        ])
        if code != 0:
            print("OpenOCD 验证失败", file=sys.stderr)
            return False
        print("OpenOCD 验证成功")
        return True

    # ── reset ──

    def reset(self):
        if self.use_openocd:
            return self._reset_openocd()
        return self._reset_pyocd()

    def _reset_openocd(self):
        code, _ = self.run_command([
            self.openocd_cmd, "-f", self._ocd_cfg,
            "-c", "init; reset; shutdown"
        ])
        if code == 0:
            print("已成功复位 MCU (OpenOCD)。")
            return True
        print("复位 MCU 失败 (OpenOCD)。", file=sys.stderr)
        return False

    def _reset_pyocd(self):
        code, _ = self.run_command([self.pyocd_cmd, "reset"])
        if code == 0:
            print("已成功复位 MCU。")
            return True
        print("复位 MCU 失败。", file=sys.stderr)
        return False

    # ── hash ──

    def _calculate_file_hash(self, file_path):
        hash_sha256 = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_sha256.update(chunk)
        return hash_sha256.hexdigest()


# ── CLI dispatch ──

def main():
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--action", required=True,
                        choices=["probe", "info", "flash", "reset", "verify", "strip"])
    parser.add_argument("--file", help="Firmware or ELF file path")
    parser.add_argument("--address", type=lambda x: int(x, 0), default=0x08000000,
                        help="Start address (default: 0x08000000)")
    parser.add_argument("--no-verify", action="store_true",
                        help="Skip verification after flash")
    args = parser.parse_args()

    dl = CMSISDAPDownloader()

    if args.action == "probe":
        info = dl.probe()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        return 1
    elif args.action == "info":
        info = dl.get_info()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        return 1
    elif args.action == "flash":
        if not args.file:
            print("flash 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if dl.flash(args.file, args.address, not args.no_verify) else 1
    elif args.action == "verify":
        if not args.file:
            print("verify 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if dl.verify(args.file, args.address) else 1
    elif args.action == "reset":
        return 0 if dl.reset() else 1
    elif args.action == "strip":
        if not args.file:
            print("strip 操作需要指定 --file", file=sys.stderr)
            return 2
        return 0 if dl.strip(args.file) else 1

    return 1


if __name__ == "__main__":
    main()
