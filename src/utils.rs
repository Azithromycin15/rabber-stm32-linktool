//! # 工具函数模块
//!
//! 这个模块提供各种工具函数，包括命令执行、工具查找和权限检查。

use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
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
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("id").arg("-u").output() {
            if output.status.success() {
                let uid = String::from_utf8_lossy(&output.stdout);
                return uid.trim() == "0";
            }
        }
        false
    }
    #[cfg(target_os = "windows")]
    {
        // 在 Windows 上，检查是否以管理员身份运行
        if let Ok(output) = Command::new("net").args(&["session"]).output() {
            output.status.success()
        } else {
            false
        }
    }
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
    #[cfg(target_os = "linux")]
    let possible_paths = [
        "/usr/bin/st-info",
        "/usr/local/bin/st-info",
        "/bin/st-info",
        "/usr/bin/stlink-info",
        "/usr/local/bin/stlink-info",
    ];
    #[cfg(target_os = "windows")]
    let possible_paths = [
        "C:\\Program Files (x86)\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
        "C:\\Program Files\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
        "ST-LINK_CLI.exe",
    ];
    find_tool("st-info", &possible_paths)
}

/// 查找 ST-Link 编程工具
///
/// 查找 st-flash 命令的路径。
pub fn find_stlink_programmer_tool() -> Option<String> {
    #[cfg(target_os = "linux")]
    let possible_paths = [
        "/usr/bin/st-flash",
        "/usr/local/bin/st-flash",
        "/bin/st-flash",
        "/usr/bin/stlink-flash",
        "/usr/local/bin/stlink-flash",
    ];
    #[cfg(target_os = "windows")]
    let possible_paths = [
        "C:\\Program Files (x86)\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
        "C:\\Program Files\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
        "ST-LINK_CLI.exe",
    ];
    find_tool("st-flash", &possible_paths)
}

/// 查找插件加载器工具
///
/// 查找 plugin-loader 二进制文件的路径。
pub fn find_project_root() -> Option<PathBuf> {
    let mut path = env::current_dir().ok()?;
    loop {
        if path.join("Cargo.toml").is_file() && path.join("plugins").is_dir() {
            return Some(path);
        }
        if !path.pop() {
            break;
        }
    }
    None
}

pub fn plugin_dir() -> PathBuf {
    if let Ok(dir) = env::var("PLUGIN_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(root) = find_project_root() {
        return root.join("plugins");
    }
    PathBuf::from("plugins")
}

pub fn manifest_path() -> PathBuf {
    if let Ok(path) = env::var("PLUGIN_MANIFEST") {
        return PathBuf::from(path);
    }
    plugin_dir().join("manifest.yaml")
}

pub fn plugin_loader_dir() -> PathBuf {
    if let Ok(dir) = env::var("PLUGIN_LOADER_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(root) = find_project_root() {
        return root.join("plugin-loader");
    }
    PathBuf::from("plugin-loader")
}

pub fn find_plugin_loader_tool() -> Option<String> {
    if let Ok(custom) = env::var("PLUGIN_LOADER_BIN") {
        if Path::new(&custom).is_file() {
            return Some(custom);
        }
    }

    let mut paths = Vec::new();
    if let Some(root) = find_project_root() {
        let root_loader = root.join("plugin-loader").join(plugin_loader_executable_name());
        paths.push(root_loader);
    }
    paths.push(PathBuf::from("plugin-loader").join(plugin_loader_executable_name()));
    paths.push(PathBuf::from("./plugin-loader").join(plugin_loader_executable_name()));
    paths.push(PathBuf::from("/usr/local/bin").join(plugin_loader_executable_name()));
    paths.push(PathBuf::from("/usr/bin").join(plugin_loader_executable_name()));

    for path in paths {
        if path.is_file() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    find_tool("plugin-loader", &["plugin-loader", "plugin-loader.exe"])
}

fn plugin_loader_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "plugin-loader.exe"
    } else {
        "plugin-loader"
    }
}

/// 自动构建 plugin-loader 二进制（如果缺失）
pub fn ensure_plugin_loader_binary() -> bool {
    if find_plugin_loader_tool().is_some() {
        return true;
    }

    let loader_dir = plugin_loader_dir();
    let output_file = loader_dir.join(plugin_loader_executable_name());
    let build_result = Command::new("go")
        .arg("build")
        .arg("-o")
        .arg(&output_file)
        .current_dir(&loader_dir)
        .status();

    match build_result {
        Ok(status) if status.success() => output_file.is_file(),
        _ => false,
    }
}

/// 打印当前环境配置
pub fn print_environment_summary() {
    let cwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let plugin_dir = plugin_dir();
    let manifest = manifest_path();
    let loader_dir = plugin_loader_dir();
    let loader_bin = env::var("PLUGIN_LOADER_BIN").unwrap_or_else(|_| {
        plugin_loader_dir().join(plugin_loader_executable_name()).to_string_lossy().to_string()
    });
    let repo_root = find_project_root();

    println!("{}", "[Env] 当前运行环境设置:".cyan());
    println!("  cwd: {}", cwd.display());
    println!("  repo_root: {}", repo_root.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "未检测到仓库根目录".to_string()));
    println!("  plugin_dir: {}", plugin_dir.display());
    println!("  manifest_path: {}", manifest.display());
    println!("  plugin_loader_dir: {}", loader_dir.display());
    println!("  plugin_loader_bin: {}", loader_bin);
}

/// 检查当前工作目录是否为仓库根目录
pub fn is_project_root() -> bool {
    Path::new("Cargo.toml").is_file() && Path::new(&plugin_dir()).is_dir()
}

/// 检查 ST-Link 工具是否已安装
///
/// 检查 st-info 和 st-flash 工具是否都可用。
pub fn check_stlink_tools_installed() -> bool {
    find_stlink_cli_tool().is_some() && find_stlink_programmer_tool().is_some()
}
