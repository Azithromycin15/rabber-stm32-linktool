# 🎛️ ST-Link V2 插件

ST-Link V2 调试器插件，用于 STM32 的 SWD 烧录与信息查询。

## 📂 目录结构

- `python/downloader.py` — Python 实现（调用 `st-info`/`st-flash`）
- `python/api_example.py` — Python API 使用示例
- `js/component.json` — 组件元数据

## ⚡ 支持的操作

| 操作 | 说明 |
|------|------|
| `probe` | 探测 ST-Link V2 设备 |
| `info` | 查询 MCU / ST-Link 信息 |
| `flash` | 烧录固件（支持自定义地址和验证） |
| `verify` | 验证已烧录的固件 |
| `reset` | 复位 MCU |

## 🚀 使用方式

```bash
# 在交互 Shell 中
stlink_v2 flash firmware.bin
stlink_v2 info
stlink_v2 reset

# 直接调用 Python
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

## 📋 元数据

- **VID**: `0x0483`
- **PID**: `0x3748`, `0x374B`
- **Flash 起始地址**: `0x08000000`
- **支持平台**: Linux、Windows
