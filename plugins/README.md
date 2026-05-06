# Plugins Directory

`plugins/` 目录用于存放插件组件定义和插件清单。

## 结构说明

- `manifest.yaml` - 插件组件注册清单
- `<component_id>/` - 单个插件组件目录
  - `python/` - Python 组件实现
  - `js/` - JS 组件元数据描述
  - `README.md` - 组件说明文档

## 插件加载流程

1. Go 组件加载器 (`plugin-loader/main.go`) 读取 `plugins/manifest.yaml`
2. 用户通过 `--component` 选择组件，并指定 `--action`
3. Go 加载器调用指定组件的 Python 实现

## 示例命令

```bash
cd plugin-loader
go run main.go --list
go run main.go --component stlink_v2 --action probe
go run main.go --component stlink_v2 --action flash --file firmware.hex
```
