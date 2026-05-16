# 🚀 rabber-stm32-linktool

> Rust-based ST-Link V2 MCU information reader and firmware flasher

## ✨ Features

- 🔍 **MCU Info** — Read STM32 chip information via SWD interface
- 🔥 **Firmware Flashing** — Supports ELF and HEX file formats
- 🧩 **Plugin Architecture** — Extensible plugin system for multiple debuggers (ST-Link V2, CMSIS-DAP)
- 💻 **Interactive Shell** — Command-line interface with built-in commands (`cd`, `pwd`, `help`)
- 🌍 **Cross-Platform** — Linux, macOS, Windows
- ⚡ **Auto Reset** — MCU automatically resets after flashing

## 📦 Build

### Dependencies

| Tool | Purpose |
|------|---------|
| Rust (cargo) | Main program compilation |
| Go | Plugin loader compilation |
| Python 3 | Plugin component runtime |
| ST-Link tools | `st-info`, `st-flash` commands |

### Build Steps

```bash
# Build all components
make build

# Build individually
make rust          # Rust main program only
make plugin-loader # Go plugin loader only
```

## 🏃 Run

Root privileges recommended for full USB access:

```bash
sudo ./target/release/rabber-stm32-linktool
```

## ⌨️ Commands

### Built-in Commands

| Command | Description |
|---------|-------------|
| `help [plugin]` | Show help info |
| `pwd` | Print current directory |
| `cd <dir>` | Change directory (supports `~`, `-`, `..`) |
| `info` | Query MCU information |
| `flash <file>` | Flash an ELF/HEX file |
| `reset` | Reset the MCU |
| `exit` / `quit` | Quit |

### Plugin Commands

Format: `<plugin_id> <command> [options]`

```
stlink_v2 flash firmware.bin
stlink_v2 info
stlink_v2 reset
```

## 🔌 Plugin System

Plugins are defined in `plugins/manifest.yaml` with three component types:

- 🐍 **Python Components** — Execution logic
- 📜 **JavaScript Components** — Metadata and interface description
- 🔧 **Go Loader** — Plugin management and dispatch

### Adding a New Plugin

1. Create a plugin directory under `plugins/`
2. Implement the Python component (`python/` directory)
3. Add JSON metadata (`js/component.json`)
4. Restart the application — plugins are auto-detected

## 📤 Release

```bash
# Create release for current platform + Windows
make release-all
```

Output in `release/` directory:
- `rabber-stm32-linktool-{version}-linux`
- `rabber-stm32-linktool-{version}-macos`
- `rabber-stm32-linktool-{version}-win64.exe`

## 🛠️ Development

```bash
make check        # Check code
make run-plugin   # Run plugin tests
make clean        # Clean build artifacts
```

## 📄 License

[MIT License](LICENSE)
