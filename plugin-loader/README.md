# Plugin Loader

`plugin-loader/main.go` 是本项目的组件加载器，它负责读取 `plugins/manifest.yaml`，根据用户指定的组件 ID 和 action 调用对应的 Python 组件。

## 编译与运行

```bash
cd plugin-loader
go build -o plugin-loader
./plugin-loader --list
./plugin-loader --component stlink_v2 --action info
./plugin-loader --component stlink_v2 --action flash --file firmware.hex
```

## 参数说明

- `--manifest`：插件清单路径，默认为 `plugins/manifest.yaml`
- `--list`：列出可用组件
- `--component`：要加载的组件 ID
- `--action`：组件执行动作，例如 `probe`、`info`、`flash`、`reset`
- `--file`：`flash` 操作使用的固件文件路径
