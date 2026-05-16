# 🧩 组件开发指南（英文） / Component Development Guide

## 📋 Overview

This project uses a **Rust core + Go loader + Python components** architecture. Each component also includes JavaScript metadata.

## 📂 Directory Structure

```
linktool/
├── Cargo.toml
├── src/                    # Rust main program
├── plugin-loader/          # Go plugin loader
├── plugins/                # Plugin directory
│   ├── manifest.yaml       # Plugin manifest (auto-generated)
│   ├── stlink_v2/          # ST-Link V2 plugin
│   │   ├── python/         # Python implementation
│   │   ├── js/             # JS metadata
│   │   └── README.md
│   ├── cmsis_dap/          # CMSIS-DAP plugin
│   └── c_compiler/         # C compiler plugin
```

## 📦 Component Format

Each component must include:

| Content | Description |
|---------|-------------|
| `python/` | Python component implementation |
| `js/component.json` | JSON metadata description |
| `README.md` | Component documentation |

## 📝 Manifest Specification

Each component in `plugins/manifest.yaml` includes:

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

## 🔧 Go Loader

`plugin-loader/main.go` reads the manifest and invokes Python components.

```bash
plugin-loader --list                                    # List all components
plugin-loader --component stlink_v2 --action info       # Run component action
plugin-loader --component stlink_v2 --action flash --file fw.bin  # Flash
```

## 🐍 Python Component Requirements

- Provide an executable script entry point
- Support `--action` parameter for operation selection
- Support standard actions: `probe`, `info`, `flash`, `reset`, `verify`
- Output errors to stderr

```bash
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex
```

## 📜 JS Component Requirements

- Export `id`, `name`, `description`, `metadata` fields
- May include an `actions` array describing supported operations
- JS modules do not execute logic — Python is the sole executable implementation

## ✍️ Creating a New Component

1. Write Python implementation in `plugins/<id>/python/`
2. Write metadata in `plugins/<id>/js/component.json`
3. Write `plugins/<id>/README.md`
4. Restart the app — components are auto-detected

## 🔮 Future Extensions

- Built-in plugin management commands in Rust framework
- More action types: `erase`, `verify`, `config`
- JS components usable as Web UI plugin description layer
