# 🔌 Plugins (English)

The `plugins/` directory contains plugin component definitions and manifest.

## 📂 Structure

- `manifest.yaml` — Plugin manifest (auto-generated on startup)
- `<component_id>/` — Individual plugin component directory
  - `python/` — Python component implementation
  - `js/component.json` — Component metadata (JSON format)
  - `README.md` — Component documentation

## 🔄 Loading Flow

1. The app auto-scans `plugins/` on startup
2. Reads each component's `js/component.json` for metadata
3. Auto-generates `manifest.yaml`
4. Go loader invokes Python components based on the manifest

## 📋 Component Definition

Each component's `js/component.json` includes:

| Field | Description |
|-------|-------------|
| `id` | Unique component identifier |
| `name` | Component name |
| `component_type` | Type (`debugger`/`tool`) |
| `python_module` | Python implementation path |
| `actions` | List of supported operations |

## 🚀 Usage Examples

```bash
# In interactive shell
stlink_v2 flash firmware.bin
cmsis_dap info
c_compiler compile main.c --mcu stm32f103c8

# Or directly via Go loader
cd plugin-loader
go run main.go --list
go run main.go --component stlink_v2 --action flash --file firmware.bin
```
