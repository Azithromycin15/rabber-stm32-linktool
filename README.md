# 🚀 rabber-stm32-linktool

> 基于 Rust 构建的 ST-Link V2 MCU 信息读取与固件烧录工具

## ✨ 特性

- 🔍 **MCU 信息读取** — 通过 SWD 接口获取 STM32 芯片信息
- 🔥 **固件烧录** — 支持 ELF 和 HEX 文件格式
- 🧩 **插件架构** — 可扩展的插件系统，支持多种调试器（ST-Link V2、CMSIS-DAP）
- 💻 **交互式 Shell** — 命令行交互界面，支持 `cd`/`pwd`/`help` 等内置命令
- 🌍 **跨平台** — Linux、macOS、Windows
- ⚡ **自动复位** — 烧录完成后自动复位 MCU

## 📦 构建

### 依赖项

| 工具 | 用途 |
|------|------|
| Rust (cargo) | 主程序编译 |
| Go | 插件加载器编译 |
| Python 3 | 插件组件运行 |
| ST-Link 工具 | `st-info`、`st-flash` 命令 |

### 构建步骤

```bash
# 构建所有组件
make build

# 分别构建
make rust          # 仅 Rust 主程序
make plugin-loader # 仅 Go 插件加载器
```

## 🏃 运行

`rabber` 不需要 root 权限启动，程序会在需要 USB 访问时自动索要单次 `sudo` 权限。

### Linux 推荐：配置 udev 规则（避免 sudo）

```bash
# 创建 udev 规则文件
sudo tee /etc/udev/rules.d/99-stlink.rules << 'EOF'
SUBSYSTEM=="usb", ATTR{idVendor}=="0483", MODE="0666"
EOF
sudo udevadm control --reload-rules
sudo udevadm trigger
```

配置后可直接运行，无需任何 sudo。

### 运行

```bash
# 交互模式
./target/release/rabber-stm32-linktool

# 直调模式
./target/release/rabber-stm32-linktool stlink_v2 flash firmware.hex
```

### 禁用自动 sudo

设置环境变量 `RABBER_NO_SUDO=1` 可完全禁用自动 sudo：

```bash
RABBER_NO_SUDO=1 ./target/release/rabber-stm32-linktool
```

## ⌨️ 交互命令

### 内置命令

| 命令 | 说明 |
|------|------|
| `help [plugin]` | 显示帮助信息 |
| `pwd` | 显示当前工作目录 |
| `cd <dir>` | 切换目录（支持 `~`、`-`、`..`） |
| `info` | 查询 MCU 信息 |
| `flash <file>` | 烧录 ELF/HEX 文件 |
| `reset` | 复位 MCU |
| `exit` / `quit` | 退出 |

### 插件命令

格式：`<插件ID> <命令> [选项]`

```
stlink_v2 flash firmware.bin
stlink_v2 info
stlink_v2 reset
```

## 🔌 插件系统

插件通过 `plugins/manifest.yaml` 定义，支持三种组件：

- 🐍 **Python 组件** — 实际执行逻辑
- 📜 **JavaScript 组件** — 元数据与接口描述
- 🔧 **Go 加载器** — 插件管理与分发

### 添加新插件

1. 在 `plugins/` 目录下创建插件目录
2. 实现 Python 组件（`python/` 目录）
3. 添加 JSON 元数据（`js/component.json`）
4. 重启应用即可自动探测

## 📤 发布

```bash
# 创建当前平台 + Windows 发布版本
make release-all
```

产物位于 `release/` 目录：
- `rabber-stm32-linktool-{version}-linux`
- `rabber-stm32-linktool-{version}-macos`
- `rabber-stm32-linktool-{version}-win64.exe`

## 🛠️ 开发

```bash
make check        # 检查代码
make run-plugin   # 运行插件测试
make clean        # 清理构建文件
```

## 📄 许可证

[MIT License](LICENSE)
