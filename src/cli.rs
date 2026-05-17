//! # CLI 参数定义
//!
//! 手动解析命令行参数，clap 仅用于生成帮助文本。
//!
//! ## 两种模式
//! 1. 交互模式: `rabber` (无参数)
//! 2. 直调模式: `rabber <插件ID> <命令> [参数...]`
//!
//! 示例:
//!   rabber stlink_v2 flash firmware.hex
//!   rabber stlink_v2 flash firmware.hex --no-verify
//!   rabber stlink_v2 info
//!   rabber stlink_v2 reset
//!   rabber c_compiler compile main.c --mcu stm32f103c8

use clap::{arg, Command};

/// 解析原始命令行参数，跳过程序名。
/// 返回 `(plugin_id, command, extra_args)` 或 `None`（交互模式）。
pub fn parse_cli() -> Option<(String, String, Vec<String>)> {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    if raw.is_empty() {
        return None; // 交互模式
    }

    // --help / -h
    if raw.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(0);
    }

    // --version / -V
    if raw.iter().any(|a| a == "--version" || a == "-V") {
        println!("rabber {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    if raw.len() < 2 {
        eprintln!("错误: 需要 <插件ID> <命令>");
        eprintln!("用法: rabber <插件ID> <命令> [参数...]");
        eprintln!("示例: rabber stlink_v2 flash firmware.hex");
        std::process::exit(1);
    }

    let plugin_id = raw[0].clone();
    let command = raw[1].clone();
    let extra_args = raw[2..].to_vec();

    Some((plugin_id, command, extra_args))
}

/// 打印帮助信息 (使用 clap builder)
fn print_help() {
    let mut cmd = Command::new("rabber")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("基于 Rust 构建的 ST-Link V2 MCU 信息读取与固件烧录工具")
        .arg(
            arg!(<plugin_id> "插件ID (例如 stlink_v2, cmsis_dap, c_compiler)")
                .required(false),
        )
        .arg(
            arg!(<command> "插件命令 (例如 flash, info, reset, compile)")
                .required(false),
        )
        .arg(
            arg!([extra_args]... "传递给插件的额外参数 (文件路径、选项等)")
                .allow_hyphen_values(true),
        )
        .after_help(
            "无参数时进入交互式 Shell。\n\
             \n\
             示例:\n  \
             rabber stlink_v2 flash firmware.hex\n  \
             rabber stlink_v2 flash firmware.hex --no-verify\n  \
             rabber stlink_v2 info\n  \
             rabber c_compiler compile main.c --mcu stm32f103c8",
        );

    cmd.print_help().unwrap();
}
