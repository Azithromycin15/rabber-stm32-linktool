use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Apt,
    Pacman,
    Dnf,
    Zypper,
    Unknown,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt-get",
            PackageManager::Pacman => "pacman",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Unknown => "unknown",
        }
    }
}

pub fn prompt_install_stlink_tools() -> bool {
    print!("{} ", "是否现在尝试安装 stlink-tools 依赖包？[Y/n]".yellow());
    io::stdout().flush().ok();

    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return false;
    }

    matches!(answer.trim().to_lowercase().as_str(), "" | "y" | "yes")
}

pub fn install_stlink_tools() -> bool {
    let pm = detect_package_manager();
    let (command, args) = match pm {
        PackageManager::Apt => ("apt-get", vec!["install", "-y", "stlink-tools"]),
        PackageManager::Pacman => ("pacman", vec!["-S", "--noconfirm", "stlink"]),
        PackageManager::Dnf => ("dnf", vec!["install", "-y", "stlink"]),
        PackageManager::Zypper => ("zypper", vec!["install", "-y", "stlink"]),
        PackageManager::Unknown => {
            println!("{}", "无法识别当前 Linux 发行版或包管理器。".red());
            return false;
        }
    };

    let distro = detect_linux_distro().unwrap_or_else(|| "未知发行版".to_string());
    println!(
        "{}",
        format!(
            "当前操作系统: {}，准备使用 {} 安装 stlink 工具...",
            distro,
            pm.name()
        )
        .cyan()
    );

    let mut install_cmd = if is_root() {
        Command::new(command)
    } else {
        let mut cmd = Command::new("sudo");
        cmd.arg(command);
        cmd
    };
    let install_cmd = install_cmd.args(args.iter());

    match install_cmd.status() {
        Ok(status) if status.success() => {
            println!("{}", "stlink-tools 安装成功。".green());
            true
        }
        Ok(status) => {
            println!("{}", format!("安装失败，退出码 {}。", status.code().unwrap_or(-1)).red());
            false
        }
        Err(err) => {
            println!("{}", format!("无法启动安装命令: {}", err).red());
            false
        }
    }
}

pub fn detect_package_manager() -> PackageManager {
    if find_executable("apt-get") {
        return PackageManager::Apt;
    }
    if find_executable("pacman") {
        return PackageManager::Pacman;
    }
    if find_executable("dnf") {
        return PackageManager::Dnf;
    }
    if find_executable("zypper") {
        return PackageManager::Zypper;
    }
    PackageManager::Unknown
}

fn find_executable(name: &str) -> bool {
    which::which(name).is_ok()
}

pub fn detect_linux_distro() -> Option<String> {
    let os_release = read_os_release()?;
    if let Some(pretty) = os_release.get("PRETTY_NAME") {
        return Some(pretty.clone());
    }
    if let Some(name) = os_release.get("NAME") {
        return Some(name.clone());
    }
    None
}

fn read_os_release() -> Option<HashMap<String, String>> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(idx) = line.find('=') {
            let key = &line[..idx];
            let mut value = line[idx + 1..].trim().to_string();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                value = value[1..value.len() - 1].to_string();
            }
            map.insert(key.to_string(), value);
        }
    }
    Some(map)
}

fn is_root() -> bool {
    match Command::new("id").arg("-u").output() {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).trim() == "0",
        _ => false,
    }
}
