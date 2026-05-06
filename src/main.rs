//! # rabber-stm32-linktool 主程序
//!
//! 这个模块包含应用程序的入口点和初始化逻辑。

mod cli;
mod flash;
mod install;
mod output;
mod plugin;
mod shell;
mod stlink;
mod utils;

use clap::Parser;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

use cli::Args;
use install::{install_stlink_tools, prompt_install_stlink_tools};
use output::{print_banner, print_mcu_info, print_stlink_info};
use plugin::PluginManager;
use shell::interactive_mode;
use stlink::{detect_stlink_by_usb, get_mcu_info_via_swd, get_stlink_info};
use utils::{check_stlink_tools_installed, is_root};

/// 应用程序的主入口点
///
/// 执行初始化检查，包括插件加载、权限检查、工具链验证、
/// USB 设备检测和 MCU 信息读取，然后进入交互模式。
fn main() {
    let _args = Args::parse();

    println!();
    print_banner();

    // 加载插件组件
    println!("{}", "[*] 加载插件组件...".cyan());
    let plugin_manager = PluginManager::load_from("plugins/manifest.yaml");
    if let Some(manager) = &plugin_manager {
        manager.list_components();
    } else {
        println!("{}", "未找到插件清单，将使用默认内置配置。".yellow());
    }

    // 检查 root 权限
    if !is_root() {
        println!("{}", "[!] 建议以 root 权限运行以获得完整 USB 访问权限".yellow());
    }

    // 检查 ST-Link 工具链
    print!("{}", "[*] 检查 ST-Link 工具链...".cyan());
    io::stdout().flush().ok();
    if !check_stlink_tools_installed() {
        println!(" {}", "未找到".red());
        if prompt_install_stlink_tools() {
            if install_stlink_tools() && check_stlink_tools_installed() {
                println!("{}", "[✓] stlink-tools 已安装，继续执行。".green());
            } else {
                println!("{}", "[✗] stlink-tools 安装失败，请手动安装后重试。".red());
                return;
            }
        } else {
            println!("{}", "已取消安装。请先手动安装 stlink-tools 再运行本程序。".yellow());
            return;
        }
    }
    println!(" {}", "已安装".green());

    // 扫描 USB 设备
    print!("{}", "[*] 扫描 USB 设备...".cyan());
    io::stdout().flush().ok();
    if !detect_stlink_by_usb() {
        println!(" {}", "未检测到 ST-Link 设备".red());
        println!("{}", "[!] 尝试列出所有 USB 设备...".yellow());
        let _ = Command::new("sh")
            .arg("-c")
            .arg("lsusb | grep -i stm32 || lsusb | grep -i st-link")
            .status();
        return;
    }
    println!(" {}", "检测到 ST-Link 设备".green());

    // 获取并显示 ST-Link 信息
    let stlink_info = get_stlink_info();
    print_stlink_info(&stlink_info);

    // 尝试通过 SWD 读取 MCU 信息
    println!("\n{}", "[*] 尝试通过 SWD 读取 MCU 信息...".cyan());
    let mcu_info = get_mcu_info_via_swd();
    if !mcu_info.chip_id.is_empty() {
        println!(" {}", "成功".green());
        print_mcu_info(&mcu_info);
    } else {
        println!(" {}", "失败".red());
        println!("{}", "[!] 可能的原因:" .yellow());
        println!("    1. 目标板未正确连接或未供电");
        println!("    2. SWD 接口连接错误");
        println!("    3. 目标 MCU 处于休眠/复位状态");
        println!("    4. SWD 接口被禁用（尝试先擦除 Flash）");
    }

    // 进入交互模式
    println!("\n{}", "[✓] 检测完成，进入交互模式".green());
    println!("{}", "[!] 提示: 可以使用 'help' 或 'help <plugin_id>' 获取支持的指令信息".yellow());

    interactive_mode(plugin_manager);
}
