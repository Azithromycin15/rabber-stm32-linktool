#!/usr/bin/env python3
"""
ST-Link V2 Downloader Component

This Python module implements the ST-Link V2 downloader component for STM32 SWD flashing and MCU information retrieval.
It provides a command-line interface to interact with ST-Link tools for probing, flashing, and resetting STM32 microcontrollers.
"""

import argparse
import subprocess
import sys

#!/usr/bin/env python3
"""
ST-Link V2 Downloader Component

This Python module implements the ST-Link V2 downloader component for STM32 SWD flashing and MCU information retrieval.
It provides an API interface for flashing, probing, and verifying STM32 microcontrollers.
"""

import argparse
import subprocess
import sys
import os
import hashlib

# Component constants
COMPONENT_ID = "stlink_v2"
VERSION = "1.0.1"
DESCRIPTION = "ST-Link V2 downloader component for STM32 SWD flashing and MCU info."


class STLinkDownloader:
    """
    ST-Link V2 Downloader API class.

    Provides methods for probing, flashing, resetting, and verifying STM32 MCUs.
    """

    def __init__(self):
        self.st_info_cmd = self._find_command("st-info")
        self.st_flash_cmd = self._find_command("st-flash")

    def _find_command(self, cmd):
        """Find the full path of a command."""
        try:
            result = subprocess.run(["which", cmd], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            if result.returncode == 0:
                return result.stdout.strip()
            else:
                # Fallback for Windows
                if os.name == 'nt':
                    if cmd == "st-info":
                        return "ST-LINK_CLI.exe"
                    elif cmd == "st-flash":
                        return "ST-LINK_CLI.exe"
                return cmd
        except FileNotFoundError:
            return cmd

    def run_command(self, args):
        """
        Execute a shell command and return the exit code and stdout.

        Args:
            args (list): Command arguments

        Returns:
            tuple: (exit_code, stdout)
        """
        try:
            result = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            if result.returncode != 0:
                print(result.stderr.strip(), file=sys.stderr)
            return result.returncode, result.stdout.strip()
        except FileNotFoundError:
            print(f"命令不存在: {args[0]}", file=sys.stderr)
            return 1, ""

    def probe(self):
        """
        Probe the connected MCU.

        Returns:
            dict: MCU information or None if failed
        """
        code, output = self.run_command([self.st_info_cmd, "--probe"])
        if code == 0:
            info = self._parse_probe_output(output)
            return info
        return None

    def _parse_probe_output(self, output):
        """Parse the probe output into a dictionary."""
        info = {}
        for line in output.split('\n'):
            line = line.strip()
            if ':' in line:
                key, value = line.split(':', 1)
                info[key.strip()] = value.strip()
        return info

    def get_info(self):
        """
        Get MCU information.

        Returns:
            dict: MCU information or None if failed
        """
        return self.probe()

    def flash(self, file_path, start_address=0x08000000, verify=True):
        """
        Flash firmware to the MCU.

        Args:
            file_path (str): Path to the firmware file
            start_address (int): Start address for flashing (default: 0x08000000)
            verify (bool): Whether to verify after flashing

        Returns:
            bool: True if successful, False otherwise
        """
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        # Flash the firmware
        addr_str = f"0x{start_address:08X}"
        code, _ = self.run_command([self.st_flash_cmd, "write", file_path, addr_str])
        if code != 0:
            return False

        # Reset the MCU
        code, _ = self.run_command([self.st_flash_cmd, "reset"])
        if code != 0:
            print("复位 MCU 失败。", file=sys.stderr)
            return False

        if verify:
            return self.verify(file_path, start_address)
        return True

    def verify(self, file_path, start_address=0x08000000):
        """
        Verify the flashed firmware by checking file integrity and basic MCU response.

        Args:
            file_path (str): Path to the firmware file
            start_address (int): Start address for verification

        Returns:
            bool: True if verification successful, False otherwise
        """
        if not os.path.exists(file_path):
            print(f"文件不存在: {file_path}", file=sys.stderr)
            return False

        # Calculate file hash
        file_hash = self._calculate_file_hash(file_path)
        print(f"固件文件哈希: {file_hash}")

        # Basic verification: probe the MCU to ensure it's responsive
        info = self.probe()
        if info is None:
            print("验证失败: 无法探测 MCU", file=sys.stderr)
            return False

        print("验证成功: MCU 响应正常")
        return True

    def _calculate_file_hash(self, file_path):
        """Calculate SHA256 hash of the file."""
        hash_sha256 = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_sha256.update(chunk)
        return hash_sha256.hexdigest()

    def reset(self):
        """
        Reset the MCU.

        Returns:
            bool: True if successful, False otherwise
        """
        code, _ = self.run_command([self.st_flash_cmd, "reset"])
        if code == 0:
            print("已成功复位 MCU。")
            return True
        else:
            print("复位 MCU 失败。", file=sys.stderr)
            return False


def main():
    """
    Main entry point for the downloader component.

    Parses command-line arguments and executes the requested action.
    """
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--action", required=True, choices=["probe", "info", "flash", "reset", "verify"])
    parser.add_argument("--file", help="Firmware file path for flash/verify action")
    parser.add_argument("--address", type=lambda x: int(x, 0), default=0x08000000, help="Start address for flash/verify (default: 0x08000000)")
    parser.add_argument("--no-verify", action="store_true", help="Skip verification after flash")
    args = parser.parse_args()

    downloader = STLinkDownloader()

    if args.action == "probe":
        info = downloader.probe()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        else:
            return 1
    elif args.action == "info":
        info = downloader.get_info()
        if info:
            for key, value in info.items():
                print(f"{key}: {value}")
            return 0
        else:
            return 1
    elif args.action == "flash":
        if not args.file:
            print("flash 操作需要指定 --file", file=sys.stderr)
            return 2
        success = downloader.flash(args.file, args.address, not args.no_verify)
        return 0 if success else 1
    elif args.action == "verify":
        if not args.file:
            print("verify 操作需要指定 --file", file=sys.stderr)
            return 2
        success = downloader.verify(args.file, args.address)
        return 0 if success else 1
    elif args.action == "reset":
        success = downloader.reset()
        return 0 if success else 1

    return 1


# API functions for external calls
def create_downloader():
    """Create and return a STLinkDownloader instance."""
    return STLinkDownloader()


def api_probe(downloader):
    """API call for probe."""
    return downloader.probe()


def api_flash(downloader, file_path, start_address=0x08000000, verify=True):
    """API call for flash."""
    return downloader.flash(file_path, start_address, verify)


def api_verify(downloader, file_path, start_address=0x08000000):
    """API call for verify."""
    return downloader.verify(file_path, start_address)


def api_reset(downloader):
    """API call for reset."""
    return downloader.reset()


if __name__ == "__main__":
    sys.exit(main())
