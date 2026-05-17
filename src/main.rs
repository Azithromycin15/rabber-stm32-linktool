//! # rabber-stm32-linktool 主程序
//!
//! 初始化 → 探测插件 → 检测工具链 → 交互 Shell

mod cli;
mod install;
mod logger;
mod output;
mod plugin;
mod shell;
mod stlink;
mod utils;

use colored::*;
use std::io::{self, Write};

use cli::parse_cli;
use install::{install_stlink_tools, prompt_install_stlink_tools};
use logger::{info as log_info, init_logger, warn as log_warn};
use output::{print_banner, print_mcu_info, print_stlink_info};
use plugin::PluginManager;
use shell::interactive_mode;
use stlink::{detect_stlink_by_usb, get_mcu_info_via_swd, get_stlink_info};
use utils::{
    build_privileged_command, cargo_package_version, check_openocd_installed,
    check_stlink_tools_installed, ensure_plugin_loader_binary, find_project_root,
    find_plugin_loader_tool, is_project_root, manifest_path, plugin_dir,
    prepare_runtime_environment, print_environment_summary,
};

fn main() {
    let cli = parse_cli();
    set_project_root();
    init_logging();
    check_env();
    let (mut mgr, dl) = probe();
    check_perms();
    if !check_tools() { return; }
    detect_device();

    // 直调模式: rabber <插件ID> <命令> [参数...]
    if let Some((pid, cmd, extra_args)) = cli {
        direct_plugin_run(mgr.as_ref(), &pid, &cmd, &extra_args);
        return;
    }

    interactive_mode(&mut mgr, dl);
}

// ── 初始化 ──

fn set_project_root() {
    if let Some(r) = find_project_root() {
        let abs = r.canonicalize().unwrap_or(r);
        std::env::set_var("PROJECT_ROOT", abs.to_string_lossy().as_ref());
    }
}

fn init_logging() {
    let path = init_logger().unwrap_or_else(|e| { eprintln!("日志初始化失败: {e}"); String::new() });
    std::env::set_var("RABBER_LOG_FILE", &path);
    println!("{}", format!("[日志] {path}").cyan());
    log_info(&format!("启动, 日志: {path}"));
}

fn check_env() {
    let v = cargo_package_version().unwrap_or_else(|| "1.2.0".into());
    print_banner(&v);
    if !prepare_runtime_environment() {
        println!("{}", "[!] 环境不完整".yellow());
        log_warn("环境不完整");
    }
    print_environment_summary();
    if !is_project_root() {
        if let Some(r) = find_project_root() {
            println!("{}", format!("[!] 非仓库根目录, 已定位: {}", r.display()).yellow());
        }
    }
    if !ensure_plugin_loader_binary() {
        println!("{}", "[!] plugin-loader 不可用".yellow());
        log_warn("plugin-loader 未找到");
    }
}

// ── 插件探测 ──

fn probe() -> (Option<PluginManager>, Option<String>) {
    println!("{}", "[*] 探测插件...".cyan());
    let t0 = std::time::Instant::now();
    let mgr = PluginManager::probe_and_generate_manifest(&plugin_dir(), &manifest_path());
    let ms = t0.elapsed().as_millis();
    match mgr {
        Some(mgr) => {
            let dls = mgr.download_components();
            println!("{}", format!("[✓] {} 个组件, {} 个下载器, {} ms", mgr.count(), dls.len(), ms).green());
            if dls.is_empty() { println!("{}", "[!] 无下载插件".yellow()); }
            if mgr.ready() { mgr.list(); } else { println!("{}", "[!] 无可用组件".yellow()); }

            let st = check_stlink_tools_installed();
            let oc = check_openocd_installed();
            println!("{}", "[依赖]".cyan());
            println!("  ST-Link: {}", if st { "✓" } else { "✗" });
            println!("  OpenOCD: {}", if oc { "✓" } else { "✗" });

            let dl = choose_downloader(&dls, st, oc);
            if let Some(ref id) = dl { println!("{}", format!("[✓] 默认下载器: {id}").green()); }
            (Some(mgr), dl)
        }
        None => {
            println!("{}", "[✗] 插件探测失败".red());
            (None, None)
        }
    }
}

fn choose_downloader(dls: &[&plugin::ComponentInfo], st: bool, oc: bool) -> Option<String> {
    let stlink = dls.iter().find(|c| c.id == "stlink_v2");
    let cmsis = dls.iter().find(|c| c.id == "cmsis_dap");
    if st && oc && dls.len() > 1 {
        println!("{}", "选择默认下载器:".cyan());
        for (i, p) in dls.iter().enumerate() { println!("  {}. {} ({})", i + 1, p.name, p.id); }
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        return input.trim().parse::<usize>().ok()
            .and_then(|i| dls.get(i.saturating_sub(1)).map(|c| c.id.clone()))
            .or_else(|| dls.first().map(|c| c.id.clone()));
    }
    let fallback = if st { stlink.or(cmsis) } else if oc { cmsis.or(stlink) } else { None };
    fallback.or(dls.first()).map(|c| c.id.clone())
}

// ── 权限 & 工具链 ──

fn check_perms() {
    // 不再在启动时强制建议 sudo。
    // 需要权限的操作会通过 build_privileged_command 自动索要单次 sudo。
}

fn check_tools() -> bool {
    print!("{}", "[*] ST-Link 工具链...".cyan());
    io::stdout().flush().ok();
    if check_stlink_tools_installed() { println!(" {}", "✓"); return true; }
    println!(" {}", "✗".red());
    if !prompt_install_stlink_tools() { println!("{}", "已取消".yellow()); return false; }
    if install_stlink_tools() && check_stlink_tools_installed() {
        println!("{}", "[✓] 已安装".green());
        true
    } else {
        println!("{}", "[✗] 安装失败".red());
        false
    }
}

// ── 设备检测 ──

fn detect_device() {
    print!("{}", "[*] USB 扫描...".cyan());
    io::stdout().flush().ok();
    if detect_stlink_by_usb() {
        println!(" {}", "检测到设备".green());
        print_stlink_info(&get_stlink_info());
        let mcu = get_mcu_info_via_swd();
        if !mcu.chip_id.is_empty() { print_mcu_info(&mcu); }
    } else {
        println!(" {}", "无设备".red());
        #[cfg(target_os = "linux")] {
            println!("{}", "[!] 尝试 lsusb...".yellow());
            let _ = std::process::Command::new("sh").arg("-c").arg("lsusb|grep -i stm").status();
        }
        #[cfg(target_os = "windows")] println!("{}", "[!] 检查设备管理器".yellow());
        #[cfg(target_os = "macos")] println!("{}", "[!] system_profiler SPUSBDataType".yellow());
    }
}

// ── 直调模式 ──

/// 命令行直调模式: 通过 plugin-loader 直接执行插件命令
fn direct_plugin_run(mgr: Option<&PluginManager>, plugin_id: &str, command: &str, extra_args: &[String]) {
    let m = match mgr {
        Some(m) => m,
        None => {
            eprintln!("错误: 插件管理器不可用");
            std::process::exit(1);
        }
    };

    let component = match m.find(plugin_id) {
        Some(c) => c,
        None => {
            eprintln!("错误: 未知插件 '{}'", plugin_id);
            std::process::exit(1);
        }
    };

    if !m.has_action(plugin_id, command) {
        eprintln!("错误: 插件 '{}' 不支持命令 '{}'", plugin_id, command);
        m.help(plugin_id);
        std::process::exit(1);
    }

    let loader = match find_plugin_loader_tool() {
        Some(p) => p,
        None => {
            eprintln!("错误: plugin-loader 未找到");
            std::process::exit(1);
        }
    };

    let mut cmd = build_privileged_command(&loader);
    cmd.arg("--manifest")
        .arg(manifest_path().to_string_lossy().as_ref())
        .arg("--component")
        .arg(&component.id)
        .arg("--action")
        .arg(command);

    if command == "flash" {
        if let Some(f) = extra_args.first() {
            cmd.arg("--file").arg(f);
            if extra_args.len() > 1 {
                cmd.arg("--");
                for a in &extra_args[1..] {
                    cmd.arg(a);
                }
            }
        } else {
            eprintln!("错误: flash 需要文件路径");
            std::process::exit(1);
        }
    } else if !extra_args.is_empty() {
        cmd.arg("--");
        for a in extra_args {
            cmd.arg(a);
        }
    }

    match cmd.status() {
        Ok(s) if s.success() => {}
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}
