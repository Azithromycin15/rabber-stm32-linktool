//! # 输出显示模块
//!
//! 这个模块负责格式化输出信息，包括横幅、ST-Link 信息、MCU 信息和帮助文本。

use colored::*;
use crate::stlink::{MCUInfo, STLinkInfo};

/// 打印应用程序横幅
pub fn print_banner(version: &str) {
    println!("{}", "╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", format!("║               Rabber烧录器 v{: <17}║", version).cyan());
    println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
}

/// 打印 ST-Link 设备信息
///
/// 以格式化的方式显示 ST-Link 的版本、序列号和 VID/PID 信息。
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

/// 打印 MCU 信息
///
/// 以格式化的方式显示目标 MCU 的芯片型号、ID、内核类型和 Flash 大小。
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

/// 显示帮助信息
///
/// 打印所有可用命令及其描述，包括插件命令的调用格式。
pub fn show_help() {
    println!("{}", "可用命令:" .cyan());
    println!("  help [plugin]      显示帮助信息，传入插件 ID 可查看插件命令");
    println!("  info               查看 MCU 信息");
    println!("  flash <file>       烧录文件到 MCU 并自动剥离 ELF 调试信息 (支持 ELF 和 HEX)");
    println!("  reset              复位 MCU");
    println!("  exit/quit          退出交互模式");
    println!("\n插件命令调用格式: <插件ID> <命令> [选项]");
}
