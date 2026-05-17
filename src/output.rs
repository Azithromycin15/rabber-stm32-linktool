//! # 输出显示
//!
//! 横幅、设备信息、帮助等格式化输出。

use colored::*;
use crate::stlink::{MCUInfo, STLinkInfo};

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

pub fn print_banner(version: &str) {
    let text = format!("Rabber烧录器 v{}", version);
    let w = display_width(&text);
    let pad = 17;
    let b = "═".repeat(pad * 2 + w);
    let s = " ".repeat(pad);
    println!("{}", format!("╔{}╗", b).cyan());
    println!("{}", format!("║{}{}{}║", s, text, s).cyan());
    println!("{}", format!("╚{}╝", b).cyan());
}

pub fn print_stlink_info(info: &STLinkInfo) {
    println!("\n{}", "[ ST-Link 信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 型号:      {:<25} │", "ST-Link/V2");
    println!("  │ 版本:      {:<25} │", info.version);
    println!("  │ 序列号:    {:<25} │", if info.serial.is_empty() { "N/A" } else { &info.serial });
    println!("  │ VID/PID:   0x{:04X}/0x{:04X}             │", info.vid, info.pid);
    println!("  └──────────────────────────────────────┘");
}

pub fn print_mcu_info(info: &MCUInfo) {
    let name = if info.chip_name.is_empty() { "Unknown" } else { &info.chip_name };
    let id = if info.chip_id.is_empty() { "N/A" } else { &info.chip_id };
    let core = if info.core.is_empty() { "Unknown" } else { &info.core };
    println!("\n{}", "[ MCU 信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 芯片:      {:<25} │", name);
    println!("  │ ID:        0x{:<23} │", id);
    println!("  │ 内核:      {:<25} │", core);
    println!("  │ Flash:     {:<8} KB               │", info.flash_size / 1024);
    println!("  └──────────────────────────────────────┘");
}

pub fn show_help() {
    println!("{}", "可用命令:".cyan());
    println!("  help [plugin]      显示帮助 (help plugin 列出所有插件)");
    println!("  pwd                当前目录");
    println!("  cd <dir>           切换目录 (支持 ~/-/../相对/绝对路径)");
    println!("  info               MCU 信息");
    println!("  flash <file>       烧录 ELF/HEX");
    println!("  reset              复位 MCU");
    println!("  exit/quit          退出");
    println!("\n插件命令: <插件ID> <命令> [选项]");
    println!("  plugin list/-l/-d/-r      列出/发现/刷新插件");
}
