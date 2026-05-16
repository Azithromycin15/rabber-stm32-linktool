# 🎛️ ST-Link V2 Plugin (English)

ST-Link V2 debugger plugin for STM32 SWD flashing and MCU information.

## 📂 Directory Structure

- `python/downloader.py` — Python implementation (invokes `st-info`/`st-flash`)
- `python/api_example.py` — Python API usage example
- `js/component.json` — Component metadata

## ⚡ Supported Actions

| Action | Description |
|--------|-------------|
| `probe` | Probe ST-Link V2 device |
| `info` | Query MCU / ST-Link information |
| `flash` | Flash firmware (supports custom address and verification) |
| `verify` | Verify flashed firmware |
| `reset` | Reset MCU |

## 🚀 Usage

```bash
# In interactive shell
stlink_v2 flash firmware.bin
stlink_v2 info
stlink_v2 reset

# Direct Python invocation
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex --address 0x08000000
python3 plugins/stlink_v2/python/downloader.py --action probe
```

## 🐍 Python API

```python
from downloader import create_downloader, api_probe, api_flash, api_reset

downloader = create_downloader()
info = api_probe(downloader)
success = api_flash(downloader, "firmware.bin", start_address=0x08000000, verify=True)
api_reset(downloader)
```

## 📋 Metadata

- **VID**: `0x0483`
- **PID**: `0x3748`, `0x374B`
- **Flash Start Address**: `0x08000000`
- **Supported Platforms**: Linux, Windows
