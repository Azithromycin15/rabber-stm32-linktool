# 🧩 组件开发指南 / Component Development Guide

## 📋 概述

本项目采用 **Rust 框架核心 + Go 组件加载器 + Python 组件实现** 的架构，每个组件还需包含 JavaScript 元数据描述。

## 📂 目录结构

```
linktool/
├── Cargo.toml
├── src/                    # Rust 主程序
├── plugin-loader/          # Go 插件加载器
├── plugins/                # 插件目录
│   ├── manifest.yaml       # 插件清单（自动生成）
│   ├── stlink_v2/          # ST-Link V2 插件
│   │   ├── python/         # Python 实现
│   │   ├── js/             # JS 元数据
│   │   └── README.md
│   ├── cmsis_dap/          # CMSIS-DAP 插件
│   └── c_compiler/         # C 编译器插件
```

## 📦 组件格式

每个组件必须包含：

| 内容 | 说明 |
|------|------|
| `python/` | Python 组件实现 |
| `js/component.json` | JSON 元数据描述 |
| `README.md` | 组件说明文档 |

## 📝 插件清单规范

`plugins/manifest.yaml` 中每个组件包含以下字段：

```yaml
components:
  - id: stlink_v2
    name: ST-Link V2
    component_type: debugger
    description: STM32 SWD flashing component
    python_module: plugins/stlink_v2/python/downloader.py
    js_module: plugins/stlink_v2/js/component.json
    metadata:
      vendor_id: "0x0483"
      product_ids: ["0x3748", "0x374B"]
      supported_platforms: [linux, windows]
      flash_start_address: "0x08000000"
    actions:
      - name: flash
        description: Flash firmware to MCU
        args: <file> [--address <addr>]
```

## 🔧 Go 加载器

`plugin-loader/main.go` 负责读取清单并调用 Python 组件。

```bash
plugin-loader --list                            # 列出所有组件
plugin-loader --component stlink_v2 --action info   # 运行组件操作
plugin-loader --component stlink_v2 --action flash --file fw.bin  # 烧录
```

## 🐍 Python 组件要求

- 提供可执行脚本入口
- 支持 `--action` 参数选择操作
- 支持标准操作：`probe`、`info`、`flash`、`reset`、`verify`
- 错误输出到 stderr

```bash
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex
```

## 📜 JS 组件要求

- 导出 `id`、`name`、`description`、`metadata` 字段
- 可包含 `actions` 数组描述支持的操作
- JS 模块不执行实际逻辑，Python 是唯一可执行实现

## ✍️ 编写新组件

1. 在 `plugins/<id>/python/` 编写 Python 实现
2. 在 `plugins/<id>/js/component.json` 编写元数据
3. 编写 `plugins/<id>/README.md`
4. 重启应用，组件自动探测

## 🔮 未来扩展

- Rust 框架内置插件管理命令
- 支持更多 action 类型：`erase`、`verify`、`config`
- JS 组件可用于 Web UI 插件描述层
