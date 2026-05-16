//! # 安装模块
//!
//! ST-Link 工具链安装与包管理器检测。

use colored::Colorize;
use std::io::{self, Write};

/// 询问用户是否安装 stlink-tools
pub fn prompt_install_stlink_tools() -> bool {
    print!("{} ", "是否现在尝试安装 stlink-tools？[Y/n]".yellow());
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
        println!("{}", "Windows 请手动安装 ST-Link Utility:".cyan());
        println!("  https://www.st.com/en/development-tools/stsw-link004.html");
        println!("或使用 OpenOCD: https://openocd.org/");
        false
    }

    #[cfg(target_os = "macos")]
    {
        println!("{}", "macOS 请使用 Homebrew:".cyan());
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
            println!("{}", "无法识别包管理器。".red());
            return false;
        }
    };

    let distro = std::fs::read_to_string("/etc/os-release").ok()
        .and_then(|s| s.lines().find_map(|l| l.strip_prefix("PRETTY_NAME=")
            .map(|v| v.trim_matches('"').to_string())))
        .unwrap_or_else(|| "未知发行版".into());

    let pkg_name = match pkg { Pkg::Apt => "apt", Pkg::Pacman => "pacman", Pkg::Dnf => "dnf", Pkg::Zypper => "zypper" };
    println!("{}", format!("系统: {}, 使用 {} 安装 stlink...", distro, pkg_name).cyan());

    let mut install = if crate::utils::is_root() {
        Command::new(cmd)
    } else {
        let mut c = Command::new("sudo"); c.arg(cmd); c
    };
    install.args(&args);

    match install.status() {
        Ok(s) if s.success() => { println!("{}", "安装成功".green()); true }
        Ok(s) => { println!("{}", format!("安装失败, exit: {}", s.code().unwrap_or(-1)).red()); false }
        Err(e) => { println!("{}", format!("无法启动安装: {}", e).red()); false }
    }
}
