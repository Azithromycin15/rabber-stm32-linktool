# ST-Link V2 Plugin Component

## 组件概述

这个组件负责 ST-Link V2 设备的连接与烧录业务逻辑。它包含一个 Python 实现和一个 JS 元数据描述版本。

## 目录结构

- `python/downloader.py` - Python 组件实现，执行 `st-info` / `st-flash` 操作。
- `js/component.js` - JS 组件版本，导出组件元数据信息。
- `README.md` - 本组件详细说明。

## 支持操作

- `probe` - 探测 ST-Link V2 设备
- `info` - 查询 MCU / ST-Link 信息
- `flash` - 烧录固件并复位 MCU
- `reset` - 复位 MCU

## 运行示例

```bash
python3 plugins/stlink_v2/python/downloader.py --action probe
python3 plugins/stlink_v2/python/downloader.py --action info
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex
python3 plugins/stlink_v2/python/downloader.py --action reset
```

## 组件元数据

该组件元数据由 `plugins/manifest.json` 和 `plugins/stlink_v2/js/component.js` 共同描述，定义了组件 ID、名称、支持平台、设备 ID、以及 Python 模块路径。
