//! # rabber-stm32-linktool 主程序
//!
//! 这个模块包含应用程序的入口点和初始化逻辑。

mod cli;
mod install;
mod logger;
mod output;
mod plugin;
mod shell;
mod stlink;
mod utils;

use clap::Parser;
use colored::*;
use std::io::{self, Write};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;

use cli::Args;
use install::{install_stlink_tools, prompt_install_stlink_tools};
use logger::{info as log_info, init_logger, warn as log_warn};
use output::{print_banner, print_mcu_info, print_stlink_info};
use plugin::PluginManager;
use shell::interactive_mode;
use stlink::{detect_stlink_by_usb, get_mcu_info_via_swd, get_stlink_info};
use utils::{check_openocd_installed, check_stlink_tools_installed, cargo_package_version, ensure_plugin_loader_binary, prepare_runtime_environment, find_project_root, is_project_root, is_root, manifest_path, print_environment_summary, plugin_dir};

/// 应用程序的主入口点
///
/// 执行初始化检查，包括插件加载、权限检查、工具链验证、
/// USB 设备检测和 MCU 信息读取，然后进入交互模式。
fn main() {
    let _args = Args::parse();

    // 保存项目根目录为绝对路径，确保 cd 后资源查找不受影响
    if let Some(root) = find_project_root() {
        if let Ok(canonical) = root.canonicalize() {
            std::env::set_var("PROJECT_ROOT", canonical.to_string_lossy().as_ref());
        } else {
            std::env::set_var("PROJECT_ROOT", root.to_string_lossy().as_ref());
        }
    }

    let log_path = init_logger().unwrap_or_else(|err| {
        eprintln!("无法初始化日志: {}", err);
        String::new()
    });
    // 设置环境变量供插件使用
    std::env::set_var("RABBER_LOG_FILE", &log_path);
    println!("{}", format!("[日志] 写入 {}", log_path).cyan());
    log_info(&format!("Application start, log file: {}", log_path));

    println!();
    let version = cargo_package_version().unwrap_or_else(|| "1.1.2".to_string());
    print_banner(&version);

    let env_ready = prepare_runtime_environment();
    if !env_ready {
        println!("{}", "[!] 当前运行环境不完整，已尝试自动创建环境，请查看日志和生成的安装脚本。".yellow());
        log_warn("Runtime environment not fully available.");
    }

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
        log_warn("plugin-loader binary could not be ensured.");
    }

    // 探测并生成插件清单
    println!("{}", "[*] 探测插件组件...".cyan());
    let start_time = std::time::Instant::now();
    let plugin_manager = PluginManager::probe_and_generate_manifest(&plugin_dir(), &manifest_path());
    let duration = start_time.elapsed();

    let mut default_downloader_id: Option<String> = None;
    if let Some(manager) = &plugin_manager {
        let download_plugins = manager.download_components();
        println!(
            "{}",
            format!(
                "[✓] 插件探测完成：{} 个组件，{} 个下载插件，耗时 {} ms",
                manager.count_components(),
                download_plugins.len(),
                duration.as_millis()
            )
            .green()
        );
        if download_plugins.is_empty() {
            println!("{}", "[!] 未发现下载插件，请检查 plugins 目录。".yellow());
        }
        if manager.is_ready() {
            manager.list_components();
        } else {
            println!("{}", "[!] 未发现可用插件组件，请检查 plugins 目录。".yellow());
        }

        let stlink_available = check_stlink_tools_installed();
        let openocd_available = check_openocd_installed();
        println!("{}", "[依赖检测]".cyan());
        println!("  ST-Link tools: {}", if stlink_available { "已安装".green() } else { "未安装".red() });
        println!("  OpenOCD: {}", if openocd_available { "已安装".green() } else { "未安装".red() });

        if !download_plugins.is_empty() {
            let stlink_plugin = download_plugins.iter().find(|c| c.id == "stlink_v2");
            let cmsis_plugin = download_plugins.iter().find(|c| c.id == "cmsis_dap");

            if stlink_available && openocd_available && download_plugins.len() > 1 {
                println!("{}", "请选择默认下载插件(输入编号):".cyan());
                for (index, plugin) in download_plugins.iter().enumerate() {
                    println!("  {}. {} ({})", index + 1, plugin.name, plugin.id);
                }
                let mut input = String::new();
                io::stdin().read_line(&mut input).ok();
                let choice = input.trim().parse::<usize>().ok().and_then(|idx| {
                    download_plugins.get(idx.saturating_sub(1)).map(|c| c.id.clone())
                });
                default_downloader_id = choice.or_else(|| download_plugins.first().map(|c| c.id.clone()));
            } else if stlink_available {
                default_downloader_id = stlink_plugin
                    .or(cmsis_plugin)
                    .or_else(|| download_plugins.first())
                    .map(|c| c.id.clone());
            } else if openocd_available {
                default_downloader_id = cmsis_plugin
                    .or(stlink_plugin)
                    .or_else(|| download_plugins.first())
                    .map(|c| c.id.clone());
            } else {
                default_downloader_id = download_plugins.first().map(|c| c.id.clone());
            }

            if let Some(selected) = &default_downloader_id {
                println!(
                    "{}",
                    format!("[✓] 默认下载插件已选定：{}", selected).green()
                );
            }
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
    let device_detected = detect_stlink_by_usb();
    if !device_detected {
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
        #[cfg(target_os = "macos")]
        {
            println!("{}", "[!] 尝试列出 USB 设备...".yellow());
            let _ = Command::new("system_profiler")
                .args(["SPUSBDataType"])
                .status();
        }
        println!("{}", "[!] 未检测到可供使用的 ST-Link 或 OpenOCD 支持的设备，将进入命令行界面。".yellow());
        println!("{}", "[!] 烧录/复位等功能将不可用，但您仍可使用 pwd/cd/插件编译等命令。".yellow());
    } else {
        println!(" {}", "检测到 ST-Link 设备".green());
    }

    // 获取并显示 ST-Link 信息
    if device_detected {
        let stlink_info = get_stlink_info();
        print_stlink_info(&stlink_info);
    }

    // 尝试通过 SWD 读取 MCU 信息
    if device_detected {
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
    } else {
        println!("{}", "\n[!] 未检测到设备，跳过 MCU 信息读取。".yellow());
    }

    // 进入交互模式
    println!("\n{}", "[✓] 检测完成，进入交互模式".green());
    println!("{}", "[!] 提示: 可以使用 'help' 或 'help <plugin_id>' 获取支持的指令信息".yellow());

    interactive_mode(plugin_manager, default_downloader_id);
}
