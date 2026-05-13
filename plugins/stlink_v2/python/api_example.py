#!/usr/bin/env python3
"""
API Usage Example for ST-Link V2 Downloader Component

This script demonstrates how to use the STLinkDownloader class API
for flashing, probing, and verifying STM32 microcontrollers.
"""

import sys
import os

# Add the downloader module to the path
sys.path.append(os.path.dirname(__file__))

from downloader import create_downloader, api_probe, api_flash, api_verify, api_reset

def main():
    print("ST-Link V2 Downloader API Example")
    print("=" * 40)

    # Create downloader instance
    downloader = create_downloader()
    print("Created STLinkDownloader instance")

    # Probe the MCU
    print("\n1. Probing MCU...")
    info = api_probe(downloader)
    if info:
        print("MCU Info:")
        for key, value in info.items():
            print(f"  {key}: {value}")
    else:
        print("Failed to probe MCU")
        return

    # Example flash (commented out to avoid accidental flashing)
    # print("\n2. Flashing firmware...")
    # firmware_file = "example_firmware.bin"  # Replace with actual file
    # if os.path.exists(firmware_file):
    #     success = api_flash(downloader, firmware_file, start_address=0x08000000, verify=True)
    #     if success:
    #         print("Firmware flashed successfully")
    #     else:
    #         print("Failed to flash firmware")
    # else:
    #     print(f"Firmware file {firmware_file} not found")

    # Example verify (commented out)
    # print("\n3. Verifying firmware...")
    # success = api_verify(downloader, firmware_file, start_address=0x08000000)
    # if success:
    #     print("Verification successful")
    # else:
    #     print("Verification failed")

    # Reset MCU
    print("\n2. Resetting MCU...")
    success = api_reset(downloader)
    if success:
        print("MCU reset successfully")
    else:
        print("Failed to reset MCU")

if __name__ == "__main__":
    main()