# rabber-stm32-linktool

由Rust构建的 ST-Link V2 MCU 信息读取与烧录工具。

### 本仓库的部分代码实现以及注释均采用AI辅助的方式进行生成编写，若对部分代码块有介意可创建issue进行反馈，这是本人第一次实际工程立项开发经验尚浅，恭迎批评指正

## 特性

- **MCU 信息读取**: 通过 SWD 接口获取 STM32 芯片信息
- **固件烧录**: 支持 ELF 和 HEX 文件格式，自动剥离调试信息
- **插件架构**: 可扩展的插件系统，支持多种调试器
- **交互式 Shell**: 提供命令行交互界面
- **跨平台支持**: Linux 和 Windows 平台
- **自动复位**: 烧录完成后自动复位 MCU

## 构建

### 依赖项

- Rust (cargo)
- Go (用于插件加载器)
- Python 3 (用于插件组件)
- ST-Link 工具 (`st-info`, `st-flash`)

### 构建步骤

```bash
# 构建所有组件
make build

# 或者分别构建
make rust          # 构建 Rust 主程序
make plugin-loader # 构建 Go 插件加载器
```

## 运行

建议使用 root 权限运行以获取完整 USB 访问权限：

```bash
sudo ./target/release/rabber-stm32-linktool
```

## 交互命令

### 内置命令

- `help` - 显示可用命令
- `info` - 查询 MCU 信息
- `flash <file>` - 烧录 ELF 或 HEX 文件，若传入 ELF 文件会自动剥离调试信息后交由插件执行烧录
- `reset` - 复位 MCU
- `exit` / `quit` - 退出交互模式

### 插件命令

插件命令格式: `<插件ID> <命令> [选项]`

例如:
- `stlink_v2 flash firmware.bin` - 使用 ST-Link V2 插件烧录固件
- `stlink_v2 info` - 显示 ST-Link 信息
- `stlink_v2 reset` - 复位 MCU

## 插件系统

### 插件清单

插件通过 `plugins/manifest.yaml` 文件定义，支持以下组件类型：

- **Python 组件**: 实际的执行逻辑
- **JavaScript 组件**: 元数据和描述
- **Go 加载器**: 插件管理和执行

### 添加新插件

1. 在 `plugins/` 目录下创建插件目录
2. 实现 Python 组件 (实际功能)
3. 添加 JavaScript 元数据文件
4. 更新 `plugins/manifest.yaml`
5. 重启应用程序

## 发布

```bash
# 创建 Linux 和 Windows 发布版本
make release
```

这将在 `release/` 目录下生成：
- `rabber-stm32-linktool-{version}-linux`
- `rabber-stm32-linktool-{version}-win64.exe`

## 开发

```bash
# 检查代码
make check

# 运行插件测试
make run-plugin

# 清理构建文件
make clean
```

## 许可证

[MIT License](LICENSE)
