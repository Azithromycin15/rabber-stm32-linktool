# 🔧 Plugin Loader

`plugin-loader/main.go` 是本项目的 Go 组件加载器，负责读取 `plugins/manifest.yaml` 并调用对应的 Python 组件。

## 🏗️ 编译

```bash
cd plugin-loader
go build -o plugin-loader
```

## 🚀 运行

```bash
./plugin-loader --list                                    # 列出所有组件
./plugin-loader --component stlink_v2 --action info       # 查询 ST-Link 信息
./plugin-loader --component stlink_v2 --action flash --file firmware.bin  # 烧录固件
```

## ⚙️ 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--manifest` | 插件清单路径 | `plugins/manifest.yaml` |
| `--list` | 列出所有可用组件 | — |
| `--component` | 组件 ID | `stlink_v2` |
| `--action` | 执行动作 | `info` |
| `--file` | 固件文件路径 | — |
| `--address` | 烧录起始地址 | — |
| `--no-verify` | 跳过验证 | `false` |

## 🔄 工作流程

1. 读取 `plugins/manifest.yaml` 清单
2. 根据 `--component` 查找对应组件
3. 调用 `python3` 执行组件的 Python 实现
4. 传递 `--action` 和额外参数给 Python 脚本
