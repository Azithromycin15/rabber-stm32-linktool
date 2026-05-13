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
use utils::{check_stlink_tools_installed, ensure_plugin_loader_binary, find_project_root, is_project_root, is_root, manifest_path, print_environment_summary, plugin_dir};

/// 应用程序的主入口点
///
/// 执行初始化检查，包括插件加载、权限检查、工具链验证、
/// USB 设备检测和 MCU 信息读取，然后进入交互模式。
fn main() {
    let _args = Args::parse();

    println!();
    print_banner();

    print_environment_summary();
    if !is_project_root() {
        if let Some(root) = find_project_root() {
            println!("{}", format!("[!] 当前目录不是仓库根目录，已定位到仓库根目录：{}", root.display()).yellow());
        } else {
            println!("{}", "[!] 无法定位仓库根目录，请确保在项目仓库或其子目录中运行。".yellow());
        }
    }

    // 自动尝试构建 plugin-loader 二进制
    if !ensure_plugin_loader_binary() {
        println!("{}", "[!] plugin-loader 二进制未找到，若需要插件功能请先执行 go build。".yellow());
    }

    // 探测并生成插件清单
    println!("{}", "[*] 探测插件组件...".cyan());
    let start_time = std::time::Instant::now();
    let plugin_manager = PluginManager::probe_and_generate_manifest(&plugin_dir(), &manifest_path());
    let duration = start_time.elapsed();

    if let Some(manager) = &plugin_manager {
        println!(
            "{}",
            format!("[✓] 插件探测完成：{} 个组件，耗时 {} ms", manager.count_components(), duration.as_millis()).green()
        );
        if manager.is_ready() {
            manager.list_components();
        } else {
            println!("{}", "[!] 未发现可用插件组件，请检查 plugins 目录。".yellow());
        }
    } else {
        println!("{}", "[✗] 插件探测失败，未生成 manifest.yaml。".red());
    }

    // 检查 root 权限
    if !is_root() {
        #[cfg(target_os = "linux")]
        println!("{}", "[!] 建议以 root 权限运行以获得完整 USB 访问权限".yellow());
        #[cfg(target_os = "windows")]
        println!("{}", "[!] 建议以管理员权限运行以获得完整 USB 访问权限".yellow());
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
        #[cfg(target_os = "linux")]
        {
            println!("{}", "[!] 尝试列出所有 USB 设备...".yellow());
            let _ = Command::new("sh")
                .arg("-c")
                .arg("lsusb | grep -i stm32 || lsusb | grep -i st-link")
                .status();
        }
        #[cfg(target_os = "windows")]
        {
            println!("{}", "[!] 请检查设备管理器中是否有 ST-Link 设备。".yellow());
            println!("{}", "[!] 尝试运行: Get-PnpDevice | Where-Object { $_.InstanceId -like '*USB*' }".yellow());
        }
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
