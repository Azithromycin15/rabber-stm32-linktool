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
VERSION = "1.3.2-Hotfix"

# ── MCU Database ──
# Each entry maps the lowercased model to its hardware profile.
# flash_start is always 0x08000000 for STM32.
MCU_DATABASE = {
    # ── STM32F1 (Cortex-M3) ──
    "stm32f103c8":  {"cpu": "cortex-m3", "flash_kb":   64, "ram_kb":  20, "family": "F1", "march": "armv7-m", "device": "stm32f103xb", "startup": "startup_stm32f103xb"},
    "stm32f103cb":  {"cpu": "cortex-m3", "flash_kb":  128, "ram_kb":  20, "family": "F1", "march": "armv7-m", "device": "stm32f103xb", "startup": "startup_stm32f103xb"},
    "stm32f103rb":  {"cpu": "cortex-m3", "flash_kb":  128, "ram_kb":  20, "family": "F1", "march": "armv7-m", "device": "stm32f103xb", "startup": "startup_stm32f103xb"},
    "stm32f103rc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103rd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103re":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103rf":  {"cpu": "cortex-m3", "flash_kb":  768, "ram_kb":  96, "family": "F1", "march": "armv7-m", "device": "stm32f103xg", "startup": "startup_stm32f103xg"},
    "stm32f103rg":  {"cpu": "cortex-m3", "flash_kb": 1024, "ram_kb":  96, "family": "F1", "march": "armv7-m", "device": "stm32f103xg", "startup": "startup_stm32f103xg"},
    "stm32f103vc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103vd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103ve":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103zc":  {"cpu": "cortex-m3", "flash_kb":  256, "ram_kb":  48, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103zd":  {"cpu": "cortex-m3", "flash_kb":  384, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},
    "stm32f103ze":  {"cpu": "cortex-m3", "flash_kb":  512, "ram_kb":  64, "family": "F1", "march": "armv7-m", "device": "stm32f103xe", "startup": "startup_stm32f103xe"},

    # ── STM32F4 (Cortex-M4, HW float) ──
    "stm32f405rg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f405xx", "startup": "startup_stm32f405xx"},
    "stm32f407vg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f407xx", "startup": "startup_stm32f407xx"},
    "stm32f407zg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f407xx", "startup": "startup_stm32f407xx"},
    "stm32f407ig":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f407xx", "startup": "startup_stm32f407xx"},
    "stm32f407zgt6":{"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f407xx", "startup": "startup_stm32f407xx"},
    "stm32f407vet6":{"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 192, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f407xx", "startup": "startup_stm32f407xx"},
    "stm32f411re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f411xe", "startup": "startup_stm32f411xe"},
    "stm32f411ce":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f411xe", "startup": "startup_stm32f411xe"},
    "stm32f412zg":  {"cpu": "cortex-m4", "flash_kb": 1024, "ram_kb": 256, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f412zx", "startup": "startup_stm32f412zx"},
    "stm32f413zh":  {"cpu": "cortex-m4", "flash_kb": 1536, "ram_kb": 320, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f413xx", "startup": "startup_stm32f413xx"},
    "stm32f429zi":  {"cpu": "cortex-m4", "flash_kb": 2048, "ram_kb": 256, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f429xx", "startup": "startup_stm32f429xx"},
    "stm32f446re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb": 128, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f446xx", "startup": "startup_stm32f446xx"},
    "stm32f401re":  {"cpu": "cortex-m4", "flash_kb":  512, "ram_kb":  96, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f401xe", "startup": "startup_stm32f401xe"},
    "stm32f401cc":  {"cpu": "cortex-m4", "flash_kb":  256, "ram_kb":  64, "family": "F4", "march": "armv7e-m", "fpu": "fpv4-sp-d16", "device": "stm32f401xc", "startup": "startup_stm32f401xc"},

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


# ── Cube Repository Mapping ──
CUBE_REPOS = {
    "F1": {"repo": "STMicroelectronics/STM32CubeF1", "branch": "master", "series": "STM32F1xx", "cpu_core": "cm3"},
    "F4": {"repo": "STMicroelectronics/STM32CubeF4", "branch": "master", "series": "STM32F4xx", "cpu_core": "cm4"},
    "F7": {"repo": "STMicroelectronics/STM32CubeF7", "branch": "master", "series": "STM32F7xx", "cpu_core": "cm7"},
    "H7": {"repo": "STMicroelectronics/STM32CubeH7", "branch": "master", "series": "STM32H7xx", "cpu_core": "cm7"},
    "G0": {"repo": "STMicroelectronics/STM32CubeG0", "branch": "master", "series": "STM32G0xx", "cpu_core": "cm0plus"},
    "G4": {"repo": "STMicroelectronics/STM32CubeG4", "branch": "master", "series": "STM32G4xx", "cpu_core": "cm4"},
    "L0": {"repo": "STMicroelectronics/STM32CubeL0", "branch": "master", "series": "STM32L0xx", "cpu_core": "cm0plus"},
    "L4": {"repo": "STMicroelectronics/STM32CubeL4", "branch": "master", "series": "STM32L4xx", "cpu_core": "cm4"},
}

# ── OpenOCD Target Mapping ──
# Maps STM32 family to OpenOCD target script name.
OPENOCD_TARGET_MAP = {
    "F1": "stm32f1x",
    "F4": "stm32f4x",
    "F7": "stm32f7x",
    "H7": "stm32h7x",
    "G0": "stm32g0x",
    "G4": "stm32g4x",
    "L0": "stm32l0x",
    "L4": "stm32l4x",
}


# ── Progress Bar ──

class ProgressBar:
    """Simple terminal progress bar (no external dependencies)."""

    def __init__(self, total, desc="", width=40):
        self.total = max(total, 1)
        self.current = 0
        self.desc = desc
        self.width = width
        self._draw()

    def update(self, n=1):
        self.current = min(self.current + n, self.total)
        self._draw()

    def _draw(self):
        pct = self.current / self.total
        filled = int(self.width * pct)
        bar = "\u2588" * filled + "\u2591" * (self.width - filled)
        info = f" {self.current}/{self.total}"
        print(f"\r  {self.desc} [{bar}] {pct*100:5.1f}%{info}", end="", flush=True)

    def done(self):
        self.current = self.total
        self._draw()
        print()


# ── HAL Download ──

def _cache_dir():
    """Cache directory for downloaded Cube packages."""
    d = os.path.join(os.path.expanduser("~"), ".cache", "rabber", "hal")
    os.makedirs(d, exist_ok=True)
    return d


def _download_zip(url, dest_path, desc="下载"):
    """Download a file with progress bar, returns True on success."""
    import urllib.request
    import ssl

    ctx = ssl.create_default_context()
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "rabber-mcu-init/1.3.2"})
        resp = urllib.request.urlopen(req, context=ctx, timeout=60)
    except Exception as e:
        print(f"\n[!] 连接失败: {e}")
        return False

    total = int(resp.headers.get("Content-Length", 0))
    pb = ProgressBar(total if total > 0 else 1, desc=desc)

    try:
        with open(dest_path, "wb") as f:
            chunk_size = 256 * 1024
            while True:
                chunk = resp.read(chunk_size)
                if not chunk:
                    break
                f.write(chunk)
                pb.update(len(chunk))
        pb.done()
        return True
    except Exception as e:
        print(f"\n[!] 下载失败: {e}")
        return False
    finally:
        resp.close()


def _extract_cube_files(zip_path, target_dir, series, device_file, startup_file):
    """Extract only the HAL/CMSIS files we need from the Cube zip."""
    import zipfile

    series_upper = series.upper()
    series_lower = series.lower()

    needed_prefixes = [
        f"Drivers/CMSIS/Core/Include/",
        f"Drivers/CMSIS/Device/ST/{series_upper}/",
        f"Drivers/{series_upper}_HAL_Driver/",
    ]

    extracted_count = 0
    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            all_names = zf.namelist()
            top_dir = ""
            for name in all_names:
                parts = name.split("/")
                if len(parts) >= 1 and parts[0]:
                    top_dir = parts[0] + "/"
                    break

            to_extract = set()
            for name in all_names:
                if name.endswith("/"):
                    continue
                inner = name[len(top_dir):] if name.startswith(top_dir) else name
                keep = any(inner.startswith(p) for p in needed_prefixes)
                if keep:
                    to_extract.add(name)

            pb = ProgressBar(len(to_extract), desc="提取 HAL 文件")
            for name in sorted(to_extract):
                inner = name[len(top_dir):] if name.startswith(top_dir) else name
                dest = os.path.join(target_dir, inner)
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                try:
                    with zf.open(name) as src, open(dest, "wb") as dst:
                        dst.write(src.read())
                    extracted_count += 1
                    pb.update()
                except Exception as e:
                    print(f"\n[!] 提取失败: {inner} — {e}")
            pb.done()
    except Exception as e:
        print(f"\n[!] zip 处理失败: {e}")
        return 0

    return extracted_count


def fetch_hal_library(mcu: str, info: dict, target_dir: str) -> int:
    """Download and extract the HAL library. Returns number of files extracted."""
    family = info.get("family", "F4")
    device_file = info.get("device", "stm32f407xx")
    startup_file = info.get("startup", "startup_stm32f407xx")
    repo_info = CUBE_REPOS.get(family)
    if not repo_info:
        print(f"[!] 系列 {family} 暂不支持 HAL 下载，将创建 bare-metal 模板")
        return 0

    repo = repo_info["repo"]
    branch = repo_info["branch"]
    series_upper = repo_info["series"]
    info.setdefault("series_upper", series_upper)
    series_lower = series_upper.lower()

    cache_dir = _cache_dir()
    total_count = 0

    # ── 1. Download main Cube repo zip (contains CMSIS Core + submodule list) ──
    zip_name = f"{repo.replace('/', '-')}-{branch}.zip"
    zip_path = os.path.join(cache_dir, zip_name)

    if not os.path.exists(zip_path):
        zip_url = f"https://github.com/{repo}/archive/refs/heads/{branch}.zip"
        print(f"\n[*] 下载 {repo} ({branch})...")
        if not _download_zip(zip_url, zip_path, desc="下载 Cube 包"):
            return 0
    else:
        print(f"\n[*] 使用缓存: {zip_path}")

    # Extract CMSIS Core files
    print(f"[*] 提取 CMSIS Core 到 {target_dir}/Drivers/ ...")
    count = _extract_cube_files(zip_path, target_dir, series_upper, device_file, startup_file)
    total_count += count

    # ── 2. Download CMSIS Device submodule ──
    cmsis_device_repo = f"STMicroelectronics/cmsis_device_{family.lower()}"
    dev_zip_name = f"{cmsis_device_repo.replace('/', '-')}-{branch}.zip"
    dev_zip_path = os.path.join(cache_dir, dev_zip_name)

    if not os.path.exists(dev_zip_path):
        dev_zip_url = f"https://github.com/{cmsis_device_repo}/archive/refs/heads/{branch}.zip"
        print(f"\n[*] 下载 {cmsis_device_repo} ({branch})...")
        if not _download_zip(dev_zip_url, dev_zip_path, desc="下载 CMSIS Device"):
            print("[!] CMSIS Device 下载失败，模板可能不完整")
        else:
            # Extract into target
            count2 = _extract_submodule_zip(
                dev_zip_path, target_dir,
                f"Drivers/CMSIS/Device/ST/{series_upper}",
                prefix_map={
                    f"Include/": f"Drivers/CMSIS/Device/ST/{series_upper}/Include/",
                    f"Source/Templates/": f"Drivers/CMSIS/Device/ST/{series_upper}/Source/Templates/",
                }
            )
            total_count += count2
            print(f"[✓] CMSIS Device: {count2} 个文件")
    else:
        print(f"\n[*] 使用缓存: {dev_zip_path}")
        count2 = _extract_submodule_zip(
            dev_zip_path, target_dir,
            f"Drivers/CMSIS/Device/ST/{series_upper}",
            prefix_map={
                f"Include/": f"Drivers/CMSIS/Device/ST/{series_upper}/Include/",
                f"Source/Templates/": f"Drivers/CMSIS/Device/ST/{series_upper}/Source/Templates/",
            }
        )
        total_count += count2
        print(f"[✓] CMSIS Device: {count2} 个文件")

    # ── 3. Download HAL Driver submodule ──
    hal_repo = f"STMicroelectronics/{series_lower}_hal_driver"
    hal_zip_name = f"{hal_repo.replace('/', '-')}-{branch}.zip"
    hal_zip_path = os.path.join(cache_dir, hal_zip_name)

    if not os.path.exists(hal_zip_path):
        hal_zip_url = f"https://github.com/{hal_repo}/archive/refs/heads/{branch}.zip"
        print(f"\n[*] 下载 {hal_repo} ({branch})...")
        if not _download_zip(hal_zip_url, hal_zip_path, desc="下载 HAL Driver"):
            print("[!] HAL Driver 下载失败，模板可能不完整")
        else:
            count3 = _extract_submodule_zip(
                hal_zip_path, target_dir,
                f"Drivers/{series_upper}_HAL_Driver",
                prefix_map={
                    f"Inc/": f"Drivers/{series_upper}_HAL_Driver/Inc/",
                    f"Src/": f"Drivers/{series_upper}_HAL_Driver/Src/",
                }
            )
            total_count += count3
            print(f"[✓] HAL Driver: {count3} 个文件")
    else:
        print(f"\n[*] 使用缓存: {hal_zip_path}")
        count3 = _extract_submodule_zip(
            hal_zip_path, target_dir,
            f"Drivers/{series_upper}_HAL_Driver",
            prefix_map={
                f"Inc/": f"Drivers/{series_upper}_HAL_Driver/Inc/",
                f"Src/": f"Drivers/{series_upper}_HAL_Driver/Src/",
            }
        )
        total_count += count3
        print(f"[✓] HAL Driver: {count3} 个文件")

    print(f"\n[✓] 总计提取 {total_count} 个文件")
    return total_count


def _extract_submodule_zip(zip_path, target_dir, dest_base, prefix_map):
    """Extract files from a submodule zip, remapping source prefixes to destination."""
    import zipfile
    extracted = 0
    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            all_names = zf.namelist()
            top_dir = ""
            for name in all_names:
                parts = name.split("/")
                if len(parts) >= 1 and parts[0]:
                    top_dir = parts[0] + "/"
                    break

            to_extract = set()
            for name in all_names:
                if name.endswith("/"):
                    continue
                inner = name[len(top_dir):] if name.startswith(top_dir) else name
                for src_prefix, dst_prefix in prefix_map.items():
                    if inner.startswith(src_prefix):
                        to_extract.add(name)
                        break

            pb = ProgressBar(len(to_extract), desc=f"  提取 {os.path.basename(dest_base)}")
            for name in sorted(to_extract):
                inner = name[len(top_dir):] if name.startswith(top_dir) else name
                for src_prefix, dst_prefix in prefix_map.items():
                    if inner.startswith(src_prefix):
                        rel = inner[len(src_prefix):]
                        dest = os.path.join(target_dir, dst_prefix, rel)
                        break
                else:
                    continue
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                try:
                    with zf.open(name) as src, open(dest, "wb") as dst:
                        dst.write(src.read())
                    extracted += 1
                    pb.update()
                except Exception as e:
                    print(f"\n[!] 提取失败: {inner} — {e}")
            pb.done()
    except Exception as e:
        print(f"\n[!] zip 处理失败: {e}")
        return 0
    return extracted


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

def makefile_content(mcu: str, info: dict, hal_mode: bool = False) -> str:
    """Generate Makefile for the given MCU."""
    upper = mcu.upper()
    cpu = info['cpu']
    march = info['march']
    series_upper = info.get('series_upper', 'STM32F4xx')
    device = info.get('device', 'stm32f407xx')
    startup = info.get('startup', 'startup_stm32f407xx')
    fpu = info.get('fpu', None)
    flash_kb = info['flash_kb']
    ram_kb = info['ram_kb']

    fpu_flags = f"-mfpu={fpu} -mfloat-abi=hard" if fpu else "-mfloat-abi=soft"

    linker_script = f"{upper}_FLASH.ld"

    if hal_mode:
        src_dir = "Src"
        inc_dir = "Inc"
        hal_inc_dirs = f"-IDrivers/CMSIS/Core/Include -IDrivers/CMSIS/Device/ST/{series_upper}/Include -IDrivers/{series_upper}_HAL_Driver/Inc"
        extra_srcs = f"$(wildcard Drivers/{series_upper}_HAL_Driver/Src/*.c) $(wildcard Drivers/CMSIS/Device/ST/{series_upper}/Source/Templates/*.c)"
        asm_startup = f"Drivers/CMSIS/Device/ST/{series_upper}/Source/Templates/gcc/{startup}.s"
        hal_define = f"-D{upper} -D{device.upper()} -DUSE_HAL_DRIVER"
    else:
        src_dir = "src"
        inc_dir = "include"
        hal_inc_dirs = ""
        extra_srcs = ""
        asm_startup = "$(wildcard $(SRC_DIR)/*.s)"
        hal_define = f"-D{upper}"

    mk = f"""# Makefile for {upper} {'HAL' if hal_mode else 'Bare-Metal'} Template
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
""" + (f"""# ── HAL / CMSIS (auto-downloaded from ST) ──
""" if hal_mode else "") + f"""
# ── Directories ──
BUILD_DIR   = build
PROCESS_DIR = process
SRC_DIR     = {src_dir}
INC_DIR     = {inc_dir}

# ── Source files ──
C_SRCS   = $(wildcard $(SRC_DIR)/*.c) {extra_srcs}
ASM_SRCS = {asm_startup}
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
CFLAGS += {hal_define}

# ── Includes ──
INCLUDES = -I$(INC_DIR) {hal_inc_dirs}

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
"""
    if hal_mode:
        mk += f"""# ── HAL object files ──
$(PROCESS_DIR)/%.o: Drivers/{series_upper}_HAL_Driver/Src/%.c
\t@mkdir -p $(PROCESS_DIR)
\t@echo "[CC]  $<"
\t$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(PROCESS_DIR)/%.o: Drivers/CMSIS/Device/ST/{series_upper}/Source/Templates/%.c
\t@mkdir -p $(PROCESS_DIR)
\t@echo "[CC]  $<"
\t$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

"""
    mk += """# ── Clean ──
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
    return mk


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


def _hal_conf_content(mcu: str, info: dict) -> str:
    """Generate stm32f4xx_hal_conf.h template."""
    upper = mcu.upper()
    device = info.get('device', 'stm32f407xx')
    series_upper = info.get('series_upper', 'STM32F4xx')
    modules = ["ADC","CAN","CORTEX","CRC","CRYP","DAC","DCMI","DMA","ETH","FLASH","GPIO","HASH","HCD","I2C","I2S","IRDA","IWDG","PCD","PWR","RCC","RNG","RTC","SAI","SD","SMARTCARD","SPI","TIM","UART","USART","WWDG"]
    mod_defs = "\n".join([f"#define HAL_{m}_MODULE_ENABLED" for m in modules])
    mod_incs = "\n".join([f"#ifdef HAL_{m}_MODULE_ENABLED\n  #include \"stm32f4xx_hal_{m.lower()}.h\"\n#endif" for m in modules])
    return f'''#ifndef __{series_upper}_HAL_CONF_H
#define __{series_upper}_HAL_CONF_H
#ifdef __cplusplus
extern "C" {{
#endif
{mod_defs}

/* ── Peripheral includes ── */
{mod_incs}

#define HSE_VALUE    8000000U
#define HSI_VALUE    16000000U
#define LSE_VALUE    32768U
#define LSI_VALUE    32000U
#define EXTERNAL_CLOCK_VALUE 12288000U
#define TICK_INT_PRIORITY 0x0FU
#define USE_RTOS 0U
#include "{device}.h"
#ifdef __cplusplus
}}
#endif
#endif'''


def _hal_it_header_content(mcu: str, info: dict) -> str:
    """Generate stm32f4xx_it.h header."""
    series_upper = info.get('series_upper', 'STM32F4xx')
    return f'''#ifndef __{series_upper}_IT_H
#define __{series_upper}_IT_H
#ifdef __cplusplus
extern "C" {{
#endif
void SysTick_Handler(void);
void NMI_Handler(void);
void HardFault_Handler(void);
void MemManage_Handler(void);
void BusFault_Handler(void);
void UsageFault_Handler(void);
#ifdef __cplusplus
}}
#endif
#endif'''


def _hal_it_content(mcu: str, info: dict) -> str:
    """Generate stm32f4xx_it.c (interrupt handlers)."""
    series_upper = info.get('series_upper', 'STM32F4xx')
    device = info.get('device', 'stm32f407xx')
    series_lower = series_upper.lower()
    return f'''#include "{device}.h"
#include "{series_lower}_hal_conf.h"
#include "{series_lower}_it.h"
extern void Error_Handler(void);
void SysTick_Handler(void) {{ HAL_IncTick(); }}
void NMI_Handler(void) {{ }}
void HardFault_Handler(void) {{ while (1) {{ }} }}
void MemManage_Handler(void) {{ while (1) {{ }} }}
void BusFault_Handler(void) {{ while (1) {{ }} }}
void UsageFault_Handler(void) {{ while (1) {{ }} }}'''


def _main_c_hal_content(mcu: str, info: dict) -> str:
    """Generate main.c using STM32 HAL library."""
    upper = mcu.upper()
    device = info.get('device', 'stm32f407xx')
    series_lower = info.get('series_upper', 'STM32F4xx').lower()

    return f'''/**
 * {upper} HAL Template — main.c
 * Generated by rabber mcu_init v{VERSION}
 *
 * Blinks on-board LED (PC13) using STM32 HAL library.
 */

#include "{series_lower}_hal.h"

static void SystemClock_Config(void);
static void MX_GPIO_Init(void);
void Error_Handler(void);

int main(void)
{{
    HAL_Init();
    SystemClock_Config();
    MX_GPIO_Init();

    while (1)
    {{
        HAL_GPIO_TogglePin(GPIOC, GPIO_PIN_13);
        HAL_Delay(500);
    }}
}}

static void SystemClock_Config(void)
{{
    RCC_OscInitTypeDef RCC_OscInitStruct = {{0}};
    RCC_ClkInitTypeDef RCC_ClkInitStruct = {{0}};

    __HAL_RCC_PWR_CLK_ENABLE();
    __HAL_PWR_VOLTAGESCALING_CONFIG(PWR_REGULATOR_VOLTAGE_SCALE1);

    RCC_OscInitStruct.OscillatorType = RCC_OSCILLATORTYPE_HSI;
    RCC_OscInitStruct.HSIState = RCC_HSI_ON;
    RCC_OscInitStruct.HSICalibrationValue = RCC_HSICALIBRATION_DEFAULT;
    RCC_OscInitStruct.PLL.PLLState = RCC_PLL_ON;
    RCC_OscInitStruct.PLL.PLLSource = RCC_PLLSOURCE_HSI;
    RCC_OscInitStruct.PLL.PLLM = 8;
    RCC_OscInitStruct.PLL.PLLN = 168;
    RCC_OscInitStruct.PLL.PLLP = RCC_PLLP_DIV2;
    RCC_OscInitStruct.PLL.PLLQ = 4;
    if (HAL_RCC_OscConfig(&RCC_OscInitStruct) != HAL_OK) Error_Handler();

    RCC_ClkInitStruct.ClockType = RCC_CLOCKTYPE_HCLK | RCC_CLOCKTYPE_SYSCLK
                                | RCC_CLOCKTYPE_PCLK1 | RCC_CLOCKTYPE_PCLK2;
    RCC_ClkInitStruct.SYSCLKSource = RCC_SYSCLKSOURCE_PLLCLK;
    RCC_ClkInitStruct.AHBCLKDivider = RCC_SYSCLK_DIV1;
    RCC_ClkInitStruct.APB1CLKDivider = RCC_HCLK_DIV4;
    RCC_ClkInitStruct.APB2CLKDivider = RCC_HCLK_DIV2;
    if (HAL_RCC_ClockConfig(&RCC_ClkInitStruct, FLASH_LATENCY_5) != HAL_OK) Error_Handler();
}}

static void MX_GPIO_Init(void)
{{
    GPIO_InitTypeDef GPIO_InitStruct = {{0}};
    __HAL_RCC_GPIOC_CLK_ENABLE();
    GPIO_InitStruct.Pin = GPIO_PIN_13;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOC, &GPIO_InitStruct);
}}

void Error_Handler(void)
{{
    __disable_irq();
    while (1) {{ }}
}}
'''


def _hal_msp_content(mcu: str, info: dict) -> str:
    """Generate stm32f4xx_hal_msp.c (MCU Support Package)."""
    series_lower = info.get('series_upper', 'STM32F4xx').lower()
    return f'''#include "{series_lower}_hal.h"
void HAL_GPIO_MspInit(GPIO_TypeDef *GPIOx) {{
    if (GPIOx == GPIOC) {{ __HAL_RCC_GPIOC_CLK_ENABLE(); }}
}}
void HAL_RCC_MspInit(void) {{ }}'''


def openocd_cfg_content(mcu: str, info: dict) -> str:
    """Generate OpenOCD configuration file for the target MCU.

    Defaults to CMSIS-DAP interface; ST-Link alternative is provided as comments.
    """
    upper = mcu.upper()
    family = info.get('family', 'F4')
    openocd_target = OPENOCD_TARGET_MAP.get(family, 'stm32f4x')

    return f"""# OpenOCD configuration for {upper}
# Generated by rabber mcu_init v{VERSION}
#
# Default interface: CMSIS-DAP (ARM DAPLink / pyOCD compatible)
# To use ST-Link instead, comment CMSIS-DAP lines and uncomment ST-Link lines.

# ── CMSIS-DAP interface ──
source [find interface/cmsis-dap.cfg]
transport select swd

# ── ST-Link interface (alternative) ──
# source [find interface/stlink.cfg]
# transport select hla_swd

# ── Target MCU ──
source [find target/{openocd_target}.cfg]

# ── Adapter speed ──
adapter speed 1000

# ── Reset configuration ──
reset_config srst_only
"""


def create_template(mcu: str, info: dict, force: bool = False, hal_mode: bool = True) -> str | None:
    """Create template files under repos/<MCU>/."""
    repos_root = resolve_repos_root()
    target_dir = os.path.join(repos_root, mcu.upper())
    upper = mcu.upper()
    series_upper = info.get('series_upper', 'STM32F4xx')
    device = info.get('device', 'stm32f407xx')
    series_lower = series_upper.lower()

    if os.path.exists(target_dir):
        if not force:
            print(f"[!] 目录已存在: {target_dir}")
            print("    使用 --force 覆盖现有文件")
            return None
        print(f"[!] 覆盖现有目录: {target_dir}")

    if hal_mode:
        src_dir = os.path.join(target_dir, "Src")
        inc_dir = os.path.join(target_dir, "Inc")
        drivers_dir = os.path.join(target_dir, "Drivers")
        for d in [target_dir, src_dir, inc_dir, drivers_dir]:
            os.makedirs(d, exist_ok=True)

        # Download HAL/CMSIS library
        fetch_hal_library(mcu, info, target_dir)

        # Write project files
        files = {
            "Makefile": makefile_content(mcu, info, hal_mode=True),
            f"{upper}_FLASH.ld": linker_script_content(mcu, info),
            "openocd.cfg": openocd_cfg_content(mcu, info),
            os.path.join("Src", "main.c"): _main_c_hal_content(mcu, info),
            os.path.join("Inc", f"{series_lower}_hal_conf.h"): _hal_conf_content(mcu, info),
            os.path.join("Inc", f"{series_lower}_it.h"): _hal_it_header_content(mcu, info),
            os.path.join("Src", f"{series_lower}_it.c"): _hal_it_content(mcu, info),
            os.path.join("Src", f"{series_lower}_hal_msp.c"): _hal_msp_content(mcu, info),
        }
    else:
        src_dir = os.path.join(target_dir, "src")
        inc_dir = os.path.join(target_dir, "include")
        for d in [target_dir, src_dir, inc_dir]:
            os.makedirs(d, exist_ok=True)

        files = {
            "Makefile": makefile_content(mcu, info, hal_mode=False),
            f"{upper}_FLASH.ld": linker_script_content(mcu, info),
            "openocd.cfg": openocd_cfg_content(mcu, info),
            os.path.join("src", "main.c"): main_c_content(mcu, info),
        }

    for rel_path, content in files.items():
        abs_path = os.path.join(target_dir, rel_path)
        os.makedirs(os.path.dirname(abs_path), exist_ok=True)
        with open(abs_path, "w") as f:
            f.write(content)
        print(f"  [✓] {rel_path}")

    mode_str = "HAL" if hal_mode else "bare-metal"
    print(f"\n[✓] {mode_str} 模板创建完成: {target_dir}")
    print(f"    进入目录后执行 'make' 编译")
    return target_dir


# ── Action handlers ──

def action_init(args: argparse.Namespace) -> None:
    """Handle the 'init' action."""
    extra = getattr(args, 'extra', []) or []
    # Filter out leading '--' separator from plugin-loader / shell
    extra = [a for a in extra if a != '--']
    force = "--force" in extra
    hal_mode = "--bare" not in extra  # default to HAL unless --bare
    mcu_input = " ".join([a for a in extra if not a.startswith("--")]) if extra else ""

    if not mcu_input:
        print("[!] 请指定 MCU 型号")
        print("用法: mcu_init init <MCU型号> [--bare] [--force]")
        print("示例: mcu_init init stm32f407zgt6")
        print("      mcu_init init stm32f407zgt6 --bare   (bare-metal, no HAL)")
        sys.exit(1)

    model = select_mcu(mcu_input)
    if model is None:
        sys.exit(1)

    info = MCU_DATABASE[model]
    if hal_mode:
        print(f"\n[*] 模式: HAL 库 (将从 GitHub 下载 STM32Cube 包)")
    else:
        print(f"\n[*] 模式: Bare-metal (纯寄存器操作)")

    target = create_template(model, info, force=force, hal_mode=hal_mode)

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
