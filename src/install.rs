//! # 安装模块
//!
//! ST-Link 工具链安装与包管理器检测。

use colored::Colorize;
use std::io::{self, Write};
use crate::t;
#[cfg_attr(not(target_os = "linux"), allow(unused_imports))]
use crate::tfmt;

/// 询问用户是否安装 stlink-tools
pub fn prompt_install_stlink_tools() -> bool {
    print!("{} ", t!("是否现在尝试安装 stlink-tools？[Y/n]", "Try to install stlink-tools now? [Y/n]").yellow());
    io::stdout().flush().ok();
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() { return false; }
    matches!(answer.trim().to_lowercase().as_str(), "" | "y" | "yes")
}

/// 尝试自动安装 stlink-tools
pub fn install_stlink_tools() -> bool {
    #[cfg(target_os = "linux")]
    return linux_install();

    #[cfg(target_os = "windows")]
    {
        println!("{}", t!("Windows 请手动安装 ST-Link Utility:",
            "Windows: Please manually install ST-Link Utility:").cyan());
        println!("  https://www.st.com/en/development-tools/stsw-link004.html");
        println!("{}", t!("或使用 OpenOCD:", "Or use OpenOCD:").cyan());
        println!("  https://openocd.org/");
        false
    }

    #[cfg(target_os = "macos")]
    {
        println!("{}", t!("macOS 请使用 Homebrew:", "macOS: Please use Homebrew:").cyan());
        println!("  brew install stlink");
        false
    }
}

// ── Linux 包管理器安装 ──

#[cfg(target_os = "linux")]
fn linux_install() -> bool {
    use std::process::Command;

    #[derive(Debug, Clone, Copy)]
    enum Pkg { Apt, Pacman, Dnf, Zypper }

    let (pkg, cmd, args): (Pkg, &str, Vec<&str>) = {
        if which::which("apt-get").is_ok() {
            (Pkg::Apt, "apt-get", vec!["install", "-y", "stlink-tools"])
        } else if which::which("pacman").is_ok() {
            (Pkg::Pacman, "pacman", vec!["-S", "--noconfirm", "stlink"])
        } else if which::which("dnf").is_ok() {
            (Pkg::Dnf, "dnf", vec!["install", "-y", "stlink"])
        } else if which::which("zypper").is_ok() {
            (Pkg::Zypper, "zypper", vec!["install", "-y", "stlink"])
        } else {
            println!("{}", t!("无法识别包管理器。", "Cannot identify package manager.").red());
            return false;
        }
    };

    let distro = std::fs::read_to_string("/etc/os-release").ok()
        .and_then(|s| s.lines().find_map(|l| l.strip_prefix("PRETTY_NAME=")
            .map(|v| v.trim_matches('"').to_string())))
        .unwrap_or_else(|| t!("未知发行版", "Unknown distribution").into());

    let pkg_name = match pkg { Pkg::Apt => "apt", Pkg::Pacman => "pacman", Pkg::Dnf => "dnf", Pkg::Zypper => "zypper" };
    println!("{}", tfmt!("系统: {}, 使用 {} 安装 stlink...", "System: {}, using {} to install stlink...", distro, pkg_name).cyan());

    let mut install = if crate::utils::is_root() {
        Command::new(cmd)
    } else {
        let mut c = Command::new("sudo"); c.arg(cmd); c
    };
    install.args(&args);

    match install.status() {
        Ok(s) if s.success() => { println!("{}", t!("安装成功", "Installation successful").green()); true }
        Ok(s) => { println!("{}", tfmt!("安装失败, exit: {}", "Installation failed, exit: {}", s.code().unwrap_or(-1)).red()); false }
        Err(e) => { println!("{}", tfmt!("无法启动安装: {}", "Cannot start installation: {}", e).red()); false }
    }
}
