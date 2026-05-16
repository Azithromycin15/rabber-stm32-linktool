# 🔌 Plugins

`plugins/` 目录存放插件组件定义与清单。

## 📂 结构

- `manifest.yaml` — 插件清单（程序启动时自动生成）
- `<component_id>/` — 单个插件组件目录
  - `python/` — Python 组件实现
  - `js/component.json` — 组件元数据（JSON 格式）
  - `README.md` — 组件说明

## 🔄 加载流程

1. 程序启动时自动扫描 `plugins/` 目录
2. 读取每个组件的 `js/component.json` 获取元数据
3. 自动生成 `manifest.yaml` 清单
4. Go 加载器根据清单调用 Python 组件

## 📋 组件定义

每个组件的 `js/component.json` 包含：

| 字段 | 说明 |
|------|------|
| `id` | 组件唯一标识 |
| `name` | 组件名称 |
| `component_type` | 类型（`debugger`/`tool`） |
| `python_module` | Python 实现路径 |
| `actions` | 支持的操作列表 |

## 🚀 使用示例

```bash
# 在交互 Shell 中
stlink_v2 flash firmware.bin
cmsis_dap info
c_compiler compile main.c --mcu stm32f103c8

# 或直接调用 Go 加载器
cd plugin-loader
go run main.go --list
go run main.go --component stlink_v2 --action flash --file firmware.bin
```
