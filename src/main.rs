//! # rabber-stm32-linktool 主程序
//!
//! 初始化 → 探测插件 → 检测工具链 → 交互 Shell

mod cli;
mod i18n;
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
    i18n::init_lang();
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
    let path = init_logger().unwrap_or_else(|e| { eprintln!("{}: {e}", t!("日志初始化失败", "Log init failed")); String::new() });
    std::env::set_var("RABBER_LOG_FILE", &path);
    println!("{}", format!("[{}] {path}", t!("日志", "Log")).cyan());
    log_info(&format!("{} {}", t!("启动, 日志:", "Startup, log:"), path));
}

fn check_env() {
    let v = cargo_package_version();
    print_banner(&v);
    if !prepare_runtime_environment() {
        println!("{}", t!("[!] 环境不完整", "[!] Environment incomplete").yellow());
        log_warn(t!("环境不完整", "Environment incomplete"));
    }
    print_environment_summary();
    if !is_project_root() {
        if let Some(r) = find_project_root() {
            println!("{}", tfmt!("[!] 非仓库根目录, 已定位: {}", "[!] Not at repo root, located: {}", r.display()).yellow());
        }
    }
    if !ensure_plugin_loader_binary() {
        println!("{}", t!("[!] plugin-loader 不可用", "[!] plugin-loader unavailable").yellow());
        log_warn(t!("plugin-loader 未找到", "plugin-loader not found"));
    }
}

// ── 插件探测 ──

fn probe() -> (Option<PluginManager>, Option<String>) {
    println!("{}", t!("[*] 探测插件...", "[*] Probing plugins...").cyan());
    let t0 = std::time::Instant::now();
    let mgr = PluginManager::probe_and_generate_manifest(&plugin_dir(), &manifest_path());
    let ms = t0.elapsed().as_millis();
    match mgr {
        Some(mgr) => {
            let dls = mgr.download_components();
            println!("{}", tfmt!("[✓] {} 个组件, {} 个下载器, {} ms", "[✓] {} components, {} downloaders, {} ms", mgr.count(), dls.len(), ms).green());
            if dls.is_empty() { println!("{}", t!("[!] 无下载插件", "[!] No download plugins").yellow()); }
            if mgr.ready() { mgr.list(); } else { println!("{}", t!("[!] 无可用组件", "[!] No available components").yellow()); }

            let st = check_stlink_tools_installed();
            let oc = check_openocd_installed();
            println!("{}", format!("[{}]", t!("依赖", "Dependencies")).cyan());
            println!("  ST-Link: {}", if st { "✓" } else { "✗" });
            println!("  OpenOCD: {}", if oc { "✓" } else { "✗" });

            let dl = choose_downloader(&dls, st, oc);
            if let Some(ref id) = dl { println!("{}", tfmt!("[✓] 默认下载器: {}", "[✓] Default downloader: {}", id).green()); }
            (Some(mgr), dl)
        }
        None => {
            println!("{}", t!("[✗] 插件探测失败", "[✗] Plugin probe failed").red());
            (None, None)
        }
    }
}

fn choose_downloader(dls: &[&plugin::ComponentInfo], st: bool, oc: bool) -> Option<String> {
    let stlink = dls.iter().find(|c| c.id == "stlink_v2");
    let cmsis = dls.iter().find(|c| c.id == "cmsis_dap");
    if st && oc && dls.len() > 1 {
        println!("{}", t!("选择默认下载器:", "Select default downloader:").cyan());
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
    print!("{}", t!("[*] ST-Link 工具链...", "[*] ST-Link toolchain...").cyan());
    io::stdout().flush().ok();
    if check_stlink_tools_installed() { println!(" {}", "✓"); return true; }
    println!(" {}", "✗".red());
    if !prompt_install_stlink_tools() { println!("{}", t!("已取消", "Cancelled").yellow()); return false; }
    if install_stlink_tools() && check_stlink_tools_installed() {
        println!("{}", t!("[✓] 已安装", "[✓] Installed").green());
        true
    } else {
        println!("{}", t!("[✗] 安装失败", "[✗] Installation failed").red());
        false
    }
}

// ── 设备检测 ──

fn detect_device() {
    print!("{}", t!("[*] USB 扫描...", "[*] USB scan...").cyan());
    io::stdout().flush().ok();
    if detect_stlink_by_usb() {
        println!(" {}", t!("检测到设备", "Device detected").green());
        print_stlink_info(&get_stlink_info());
        let mcu = get_mcu_info_via_swd();
        if !mcu.chip_id.is_empty() { print_mcu_info(&mcu); }
    } else {
        println!(" {}", t!("无设备", "No device").red());
        #[cfg(target_os = "linux")] {
            println!("{}", t!("[!] 尝试 lsusb...", "[!] Trying lsusb...").yellow());
            let _ = std::process::Command::new("sh").arg("-c").arg("lsusb|grep -i stm").status();
        }
        #[cfg(target_os = "windows")] println!("{}", t!("[!] 检查设备管理器", "[!] Check Device Manager").yellow());
        #[cfg(target_os = "macos")] println!("{}", t!("[!] system_profiler SPUSBDataType", "[!] system_profiler SPUSBDataType").yellow());
    }
}

// ── 直调模式 ──

/// 需要将首个参数作为文件路径传给 `--file` 的 action 名称
const FILE_ACTIONS: &[&str] = &["flash", "verify", "compile", "strip"];

/// 命令行直调模式: 通过 plugin-loader 直接执行插件命令
/// `plugin_cmd` 对应用户输入的插件命令名（manifest 中的 command 字段）
fn direct_plugin_run(mgr: Option<&PluginManager>, plugin_cmd: &str, command: &str, extra_args: &[String]) {
    let m = match mgr {
        Some(m) => m,
        None => {
            eprintln!("{}: {}", t!("错误", "Error"), t!("插件管理器不可用", "Plugin manager unavailable"));
            std::process::exit(1);
        }
    };

    let component = match m.find_by_command(plugin_cmd) {
        Some(c) => c,
        None => {
            eprintln!("{}: {}", t!("未知插件", "Unknown plugin"), plugin_cmd);
            std::process::exit(1);
        }
    };

    if !m.has_action(&component.id, command) {
        eprintln!("{}: {} '{}' {}", t!("插件", "Plugin"), plugin_cmd, t!("不支持命令", "does not support command"), command);
        m.help(&component.id);
        std::process::exit(1);
    }

    let loader = match find_plugin_loader_tool() {
        Some(p) => p,
        None => {
            eprintln!("{}: {}", t!("错误", "Error"), t!("plugin-loader 未找到", "plugin-loader not found"));
            std::process::exit(1);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

    let mut cmd = build_privileged_command(&loader);
    cmd.arg("--manifest")
        .arg(manifest_path().to_string_lossy().as_ref())
        .arg("--component")
        .arg(&component.id)
        .arg("--action")
        .arg(command);

    if FILE_ACTIONS.contains(&command) {
        if let Some(f) = extra_args.first() {
            let resolved = if std::path::Path::new(f).is_relative() {
                cwd.join(f)
            } else {
                std::path::PathBuf::from(f)
            };
            cmd.arg("--file").arg(resolved.to_string_lossy().as_ref());
            if extra_args.len() > 1 {
                cmd.arg("--");
                for a in &extra_args[1..] {
                    cmd.arg(a);
                }
            }
        } else {
            eprintln!("{}: {}", t!("错误", "Error"), tfmt!("{} 需要文件路径", "{} requires a file path", command));
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
            eprintln!("{}: {}", t!("错误", "Error"), e);
            std::process::exit(1);
        }
    }
}
