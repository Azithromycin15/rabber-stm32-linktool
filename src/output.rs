use colored::*;
use crate::stlink::{MCUInfo, STLinkInfo};

pub fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           ST-Link V2 MCU 信息读取工具 v1.0           ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
}

pub fn print_stlink_info(info: &STLinkInfo) {
    println!("\n{}", "[ ST-Link 信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 型号:      {:<25} │", "ST-Link/V2");
    println!("  │ 版本:      {:<25} │", info.version);
    println!(
        "  │ 序列号:    {:<25} │",
        if info.serial.is_empty() { "N/A" } else { &info.serial }
    );
    println!(
        "  │ VID/PID:   0x{:04X}/0x{:04X}             │",
        info.vid, info.pid
    );
    println!("  └──────────────────────────────────────┘");
}

pub fn print_mcu_info(info: &MCUInfo) {
    println!("\n{}", "[ 目标MCU信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!(
        "  │ 芯片型号:  {:<25} │",
        if info.chip_name.is_empty() { "Unknown" } else { &info.chip_name }
    );
    println!(
        "  │ 芯片ID:    0x{:<23} │",
        if info.chip_id.is_empty() { "N/A" } else { &info.chip_id }
    );
    println!(
        "  │ 内核:      {:<25} │",
        if info.core.is_empty() { "Unknown" } else { &info.core }
    );
    let flash_kb = info.flash_size / 1024;
    println!("  │ Flash大小: {:<8} KB               │", flash_kb);
    println!("  └──────────────────────────────────────┘");
}

pub fn show_help() {
    println!("{}", "可用命令:" .cyan());
    println!("  help          显示此帮助信息");
    println!("  info          查看 MCU 信息");
    println!("  flash <file>  烧录文件到 MCU (支持 ELF 和 HEX)");
    println!("  elf2hex <elf> <hex>  将 ELF 转换为 HEX 格式");
    println!("  reset         复位 MCU");
    println!("  exit/quit     退出交互模式");
}
