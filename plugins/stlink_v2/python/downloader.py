#!/usr/bin/env python3
"""
ST-Link V2 Downloader Component

This Python module implements the ST-Link V2 downloader component for STM32 SWD flashing and MCU information retrieval.
It provides a command-line interface to interact with ST-Link tools for probing, flashing, and resetting STM32 microcontrollers.
"""

import argparse
import subprocess
import sys

# Component constants
COMPONENT_ID = "stlink_v2"
VERSION = "1.0.1"
DESCRIPTION = "ST-Link V2 downloader component for STM32 SWD flashing and MCU info."


def run_command(args):
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


def probe():
    """
    Probe the connected MCU using st-info --probe.

    Returns:
        int: Exit code (0 for success)
    """
    code, output = run_command(["st-info", "--probe"])
    if code == 0:
        print(output)
    return code


def get_info():
    """
    Get MCU information using st-info --probe.

    Returns:
        int: Exit code (0 for success)
    """
    code, output = run_command(["st-info", "--probe"])
    if code != 0:
        return code
    print(output)
    return 0


def flash(file_path):
    """
    Flash firmware to the MCU and reset it.

    Args:
        file_path (str): Path to the firmware file

    Returns:
        int: Exit code (0 for success)
    """
    code, _ = run_command(["st-flash", "write", file_path, "0x08000000"])
    if code != 0:
        return code
    code, output = run_command(["st-flash", "reset"])
    if code == 0:
        print("已成功复位 MCU。")
    else:
        print("复位 MCU 失败。", file=sys.stderr)
    return code


def reset():
    """
    Reset the MCU using st-flash reset.

    Returns:
        int: Exit code (0 for success)
    """
    code, _ = run_command(["st-flash", "reset"])
    if code != 0:
        return code
    print("已成功复位 MCU。")
    return 0


def main():
    """
    Main entry point for the downloader component.

    Parses command-line arguments and executes the requested action.
    """
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--action", required=True, choices=["probe", "info", "flash", "reset"])
    parser.add_argument("--file", help="Firmware file path for flash action")
    args = parser.parse_args()

    if args.action == "probe":
        return probe()
    if args.action == "info":
        return get_info()
    if args.action == "flash":
        if not args.file:
            print("flash 操作需要指定 --file", file=sys.stderr)
            return 2
        return flash(args.file)
    if args.action == "reset":
        return reset()

    return 1


if __name__ == "__main__":
    sys.exit(main())
