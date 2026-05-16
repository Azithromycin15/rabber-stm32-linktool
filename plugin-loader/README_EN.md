# 🔧 Plugin Loader (English)

`plugin-loader/main.go` is the Go component loader for this project. It reads `plugins/manifest.yaml` and invokes the corresponding Python components.

## 🏗️ Build

```bash
cd plugin-loader
go build -o plugin-loader
```

## 🚀 Usage

```bash
./plugin-loader --list                                    # List all components
./plugin-loader --component stlink_v2 --action info       # Query ST-Link info
./plugin-loader --component stlink_v2 --action flash --file firmware.bin  # Flash firmware
```

## ⚙️ Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--manifest` | Plugin manifest path | `plugins/manifest.yaml` |
| `--list` | List all available components | — |
| `--component` | Component ID | `stlink_v2` |
| `--action` | Action to execute | `info` |
| `--file` | Firmware file path | — |
| `--address` | Flash start address | — |
| `--no-verify` | Skip verification | `false` |

## 🔄 Workflow

1. Read `plugins/manifest.yaml`
2. Look up the component by `--component`
3. Invoke `python3` to run the component's Python implementation
4. Pass `--action` and extra arguments to the Python script
