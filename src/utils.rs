//! # 工具函数模块
//!
//! 这个模块提供各种工具函数，包括命令执行、工具查找和权限检查。

use std::path::Path;
use std::process::{Command, Stdio};
use which::which;

/// 命令执行结果结构体
///
/// 包含命令的退出状态码和标准输出。
pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
}

/// 检查当前用户是否为 root
///
/// 通过执行 `id -u` 命令检查用户 ID 是否为 0。
pub fn is_root() -> bool {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        if output.status.success() {
            let uid = String::from_utf8_lossy(&output.stdout);
            return uid.trim() == "0";
        }
    }
    false
}

/// 执行外部命令
///
/// 执行指定的命令和参数，返回执行结果。
pub fn execute_command(cmd: &str, args: &[&str]) -> CommandResult {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => CommandResult {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        },
        Err(_) => CommandResult {
            status: -1,
            stdout: String::new(),
        },
    }
}

/// 查找工具
///
/// 在指定的可能路径列表中查找工具，如果找不到则使用 which 命令。
pub fn find_tool(name: &str, possible_paths: &[&str]) -> Option<String> {
    for path in possible_paths {
        if Path::new(path).is_file() {
            return Some(path.to_string());
        }
    }

    if let Ok(path) = which(name) {
        return Some(path.to_string_lossy().to_string());
    }

    None
}

/// 查找 ST-Link CLI 工具
///
/// 查找 st-info 命令的路径。
pub fn find_stlink_cli_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-info",
        "/usr/local/bin/st-info",
        "/bin/st-info",
        "/usr/bin/stlink-info",
        "/usr/local/bin/stlink-info",
    ];
    find_tool("st-info", &possible_paths)
}

/// 查找 ST-Link 编程工具
///
/// 查找 st-flash 命令的路径。
pub fn find_stlink_programmer_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-flash",
        "/usr/local/bin/st-flash",
        "/bin/st-flash",
        "/usr/bin/stlink-flash",
        "/usr/local/bin/stlink-flash",
    ];
    find_tool("st-flash", &possible_paths)
}

/// 查找插件加载器工具
///
/// 查找 plugin-loader 二进制文件的路径。
pub fn find_plugin_loader_tool() -> Option<String> {
    let possible_paths = [
        "plugin-loader/plugin-loader",
        "./plugin-loader/plugin-loader",
        "/usr/local/bin/plugin-loader",
        "/usr/bin/plugin-loader",
    ];
    find_tool("plugin-loader", &possible_paths)
}

/// 检查 ST-Link 工具是否已安装
///
/// 检查 st-info 和 st-flash 工具是否都可用。
pub fn check_stlink_tools_installed() -> bool {
    find_stlink_cli_tool().is_some() && find_stlink_programmer_tool().is_some()
}
