# Component Development Guide for rabber-stm32-linktool

## 目标

本项目使用 Rust 作为框架核心，组件加载器使用 Go 编写，组件实现使用 Python 编写，且每个组件都必须包含一个 JS 元数据版本和详细的文档说明。

## 目录结构

```
linktool/
├─ Cargo.toml
├─ src/
├─ plugins/
│  ├─ manifest.yaml
│  ├─ stlink_v2/
│  │  ├─ python/downloader.py
│  │  ├─ js/component.js
│  │  └─ README.md
├─ plugin-loader/main.go
└─ COMPONENT_GUIDE.md
```

## 组件格式

每一个组件必须包含以下内容：

- `python/` 目录：Python 组件实现
- `js/` 目录：JS 元数据描述和预留接口
- `README.md`：组件说明文档
- `plugins/manifest.yaml`：插件注册清单，包含组件 ID、类型、描述、入口模块路径以及元数据

## 插件清单规范

`plugins/manifest.yaml` 中每个组件包含：

- `id`：组件唯一标识
- `name`：组件名称
- `component_type`：组件类型，例如 `debugger`、`flasher`、`board`
- `description`：组件简介
- `python_module`：组件 Python 实现入口
- `js_module`：JS 元数据入口
- `metadata`：组件自定义元数据

示例：

```yaml
components:
  - id: stlink_v2
    name: ST-Link V2
    component_type: debugger
    description: ST-Link V2 connector plugin for STM32 SWD flashing and MCU information.
    python_module: plugins/stlink_v2/python/downloader.py
    js_module: plugins/stlink_v2/js/component.js
    metadata:
      vendor_id: "0x0483"
      product_ids:
        - "0x3748"
        - "0x374B"
      supported_platforms:
        - linux
        - windows
      flash_start_address: "0x08000000"
```
## Go 组件加载器

`plugin-loader/main.go` 负责读取 `plugins/manifest.yaml`，根据组件 ID 和 action 选择对应 Python 组件运行。

### 支持命令

- `plugin-loader --list`：列出所有可用组件
- `plugin-loader --component stlink_v2 --action info`：运行指定组件的 `info` 操作
- `plugin-loader --component stlink_v2 --action flash --file firmware.hex`：运行指定组件的烧录操作

## Python 组件实现要求

Python 组件必须：

- 提供一个可执行脚本入口
- 支持通过 `--action` 参数选择操作
- 支持 `probe`, `info`, `flash`, `reset` 等标准操作
- 保持操作与系统命令隔离，并将错误输出到 stderr

示例入口参数：

```bash
python3 plugins/stlink_v2/python/downloader.py --action probe
python3 plugins/stlink_v2/python/downloader.py --action flash --file firmware.hex
```

## JS 组件实现要求

JS 组件主要用于展示组件元数据和接口描述。

- 导出 `id`, `name`, `description`, `metadata` 等字段
- 可以包含一个轻量级 `run()` 方法，作为界面层或元数据层的示例
- JS 模块不需要直接执行烧录逻辑，Python 实现为唯一可执行实现

## 编写新组件步骤

1. 在 `plugins/manifest.yaml` 中注册新的组件信息
2. 在 `plugins/<component_id>/python/` 下编写 Python 实现
3. 在 `plugins/<component_id>/js/` 下编写 JS 元数据模块
4. 编写 `plugins/<component_id>/README.md`，说明组件功能、目录结构和运行方式
5. 使用 `plugin-loader` 运行新组件，并验证返回结果

## 兼容性注意

- Python 组件默认使用 `python3`
- Go 加载器需要在本机安装 Go 编译环境，可通过 `go build` 生成二进制
- Rust 框架负责插件发现、元数据解析和用户交互入口

## 未来扩展

- 可以在 Rust 框架中直接添加插件管理命令
- 组件可以扩展更多 `action` 类型，比如 `erase`、`verify`、`config`
- JS 组件可以用作 Web UI 或 Electron 前端的插件描述层
