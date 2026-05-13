# Plugins Directory

`plugins/` 目录用于存放插件组件定义和插件清单。

## 结构说明

- `manifest.yaml` - 插件组件注册清单（由程序自动生成）
- `<component_id>/` - 单个插件组件目录
  - `python/` - Python 组件实现
  - `js/component.json` - 组件元数据描述（JSON格式）
  - `README.md` - 组件说明文档

## 插件加载流程

1. 程序启动时自动探测 `plugins/` 目录下的组件
2. 读取每个组件的 `js/component.json` 文件获取元数据
3. 动态生成 `plugins/manifest.yaml` 清单文件
4. Go 组件加载器 (`plugin-loader/main.go`) 使用生成的清单
5. 用户通过 `--component` 选择组件，并指定 `--action`
6. Go 加载器调用指定组件的 Python 实现

## 组件定义

每个组件需要提供 `js/component.json` 文件，包含：
- `plugin_name`: 插件显示名称
- `command`: 命令前缀
- `actions`: 支持的操作列表

## 示例命令

```bash
# 运行主程序（自动探测插件）
cargo run

# 使用Go加载器
cd plugin-loader
go run main.go --list
go run main.go --component stlink_v2 --action probe
go run main.go --component stlink_v2 --action flash --file firmware.hex --address 0x08000000
```
