# ST-Link V2 Plugin Component

## 组件概述

这个组件负责 ST-Link V2 设备的连接与烧录业务逻辑。它包含一个 Python 实现和一个 JS 元数据描述版本。

## 目录结构

- `python/downloader.py` - Python 组件实现，执行 `st-info` / `st-flash` 操作。
- `python/api_example.py` - Python API 使用示例。
- `js/component.js` - JS 组件版本，导出组件元数据信息。
- `README.md` - 本组件详细说明。

## 支持操作

- `probe` - 探测 ST-Link V2 设备
- `info` - 查询 MCU / ST-Link 信息
- `flash` - 烧录固件并复位 MCU（支持自定义起始地址和验证）
- `verify` - 验证已烧录的固件
- `reset` - 复位 MCU

## 命令行使用

```bash
python3 plugins/stlink_v2/python/downloader.py --action probe
python3 plugins/stlink_v2/python/downloader.py --action info
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex --address 0x08000000
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.bin --address 0x08000000 --no-verify
python3 plugins/stlink_v2/python/downloader.py --action verify --file firmware.hex --address 0x08000000
python3 plugins/stlink_v2/python/downloader.py --action reset
```

## API 使用

```python
from downloader import create_downloader, api_probe, api_flash, api_verify, api_reset

# 创建下载器实例
downloader = create_downloader()

# 探测MCU
info = api_probe(downloader)
if info:
    print("MCU Info:", info)

# 烧录固件（起始地址0x08000000，带验证）
success = api_flash(downloader, "firmware.bin", start_address=0x08000000, verify=True)

# 验证固件
success = api_verify(downloader, "firmware.bin", start_address=0x08000000)

# 复位MCU
success = api_reset(downloader)
```

运行API示例：
```bash
python3 plugins/stlink_v2/python/api_example.py
```

## 组件元数据

该组件元数据由 `plugins/manifest.yaml` 和 `plugins/stlink_v2/js/component.js` 共同描述，定义了组件 ID、名称、支持平台、设备 ID、以及 Python 模块路径。
