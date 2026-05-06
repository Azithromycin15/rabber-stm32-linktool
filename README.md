# rabber-stm32-linktool

由Rust构建的 ST-Link V2 MCU 信息读取与烧录工具。（开发中）

## 构建

```bash
cargo build --release
```

## 运行

建议使用 root 权限运行以获取完整 USB 访问权限：

```bash
sudo ./target/release/rabber-stm32-linktool
```

## 交互命令

- `help` - 显示可用命令
- `info` - 查询 MCU 信息
- `flash <file>` - 烧录 ELF 或 HEX 文件
- `elf2hex <elf> <hex>` - 将 ELF 文件转换为 Intel HEX
- `reset` - 复位 MCU
- `exit` / `quit` - 退出交互模式
