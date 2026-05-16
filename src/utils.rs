//! # 工具函数模块
//!
//! 这个模块提供各种工具函数，包括命令执行、工具查找和权限检查。

use colored::Colorize;
use crate::logger::{info as log_info, warn as log_warn};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use reqwest::blocking;
use zip;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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
    #[cfg(target_os = "macos")]
    {
        // 在 macOS 上，检查是否以 root 身份运行
        match Command::new("id").arg("-u").output() {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).trim() == "0",
            _ => false,
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
/// Linux 上查找 st-info；Windows 上查找 ST-LINK_CLI.exe。
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
    #[cfg(target_os = "macos")]
    let possible_paths = [
        "/usr/local/bin/st-info",
        "/usr/bin/st-info",
        "/bin/st-info",
        "/usr/local/bin/stlink-info",
        "/usr/bin/stlink-info",
    ];
    #[cfg(target_os = "linux")]
    {
        find_tool("st-info", &possible_paths)
    }
    #[cfg(target_os = "windows")]
    {
        find_tool("ST-LINK_CLI.exe", &possible_paths)
    }
    #[cfg(target_os = "macos")]
    {
        find_tool("st-info", &possible_paths)
    }
}

/// 查找 ST-Link 编程工具
///
/// Linux 上查找 st-flash；Windows 上查找 ST-LINK_CLI.exe。
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
    #[cfg(target_os = "macos")]
    let possible_paths = [
        "/usr/local/bin/st-flash",
        "/usr/bin/st-flash",
        "/bin/st-flash",
        "/usr/local/bin/stlink-flash",
        "/usr/bin/stlink-flash",
    ];
    #[cfg(target_os = "linux")]
    {
        find_tool("st-flash", &possible_paths)
    }
    #[cfg(target_os = "windows")]
    {
        find_tool("ST-LINK_CLI.exe", &possible_paths)
    }
    #[cfg(target_os = "macos")]
    {
        find_tool("st-flash", &possible_paths)
    }
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

pub fn logs_dir() -> PathBuf {
    if let Some(root) = find_project_root() {
        return root.join("logs");
    }
    PathBuf::from("logs")
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

/// 获取 Cargo.toml 里的包版本
pub fn cargo_package_version() -> Option<String> {
    let content = fs::read_to_string("Cargo.toml").ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

/// 检查 OpenOCD 是否已安装
pub fn check_openocd_installed() -> bool {
    let possible_paths = [
        "openocd",
        "openocd.exe",
        "/usr/bin/openocd",
        "/usr/local/bin/openocd",
        "C:\\Program Files (x86)\\OpenOCD\\bin\\openocd.exe",
        "C:\\Program Files\\OpenOCD\\bin\\openocd.exe",
    ];
    find_tool("openocd", &possible_paths).is_some()
}

/// 检查 Go 是否已安装
pub fn check_go_installed() -> bool {
    let possible_paths = ["go", "go.exe"];
    find_tool("go", &possible_paths).is_some()
}

/// 判断当前 IP 是否为中国大陆 IP
pub fn is_china_ip() -> Option<bool> {
    if let Some(curl) = find_tool("curl", &["curl", "curl.exe"]) {
        let result = Command::new(curl)
            .args(["-s", "https://ipinfo.io/country"]).output();
        if let Ok(output) = result {
            if output.status.success() {
                let country = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(country.eq_ignore_ascii_case("CN"));
            }
        }
    }
    if let Some(wget) = find_tool("wget", &["wget", "wget.exe"]) {
        let result = Command::new(wget)
            .args(["-qO-", "https://ipinfo.io/country"]).output();
        if let Ok(output) = result {
            if output.status.success() {
                let country = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(country.eq_ignore_ascii_case("CN"));
            }
        }
    }
    None
}

const PLUGIN_LOADER_GITEE_REPO: &str = "https://gitee.com/azithromycin/rabber-stm32-linktool-plugin-loader.git";
const PLUGIN_LOADER_GITHUB_REPO: &str = "https://github.com/azithromycin/rabber-stm32-linktool-plugin-loader.git";

fn select_plugin_loader_repo() -> &'static str {
    match is_china_ip() {
        Some(true) => PLUGIN_LOADER_GITEE_REPO,
        _ => PLUGIN_LOADER_GITHUB_REPO,
    }
}

/// 下载并解压ZIP文件到指定目录
#[cfg(not(target_os = "macos"))]
fn download_file(url: &str, dest: &Path) -> bool {
    let response = match blocking::get(url) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };

    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(dest, &bytes).is_ok()
}

fn download_and_extract_zip(url: &str, dest: &Path) -> bool {
    let response = match blocking::get(url) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };

    let zip_path = dest.join("download.zip");
    if let Some(parent) = zip_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let write_result = fs::write(&zip_path, &bytes);
    if write_result.is_err() {
        let _ = fs::remove_file(&zip_path);
        return false;
    }

    let extracted = extract_zip_file(&zip_path, dest);
    let _ = fs::remove_file(&zip_path);
    extracted
}

/// 解压ZIP文件到指定目录
fn extract_zip_file(zip_path: &Path, dest: &Path) -> bool {
    let file = match fs::File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let outpath = dest.join(file.name());
        if file.name().ends_with('/') {
            let _ = fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                let _ = fs::create_dir_all(p);
            }
            let mut outfile = match fs::File::create(&outpath) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let _ = std::io::copy(&mut file, &mut outfile);
        }
    }
    true
}

/// 下载 plugin-loader 可执行文件到 plugins 目录
pub fn ensure_plugin_loader_source() -> bool {
    let plugin_path = plugin_dir().join(plugin_loader_executable_name());
    if plugin_path.is_file() {
        return true;
    }

    // macOS 没有预编译二进制，需要本地 Go 编译
    #[cfg(target_os = "macos")]
    {
        println!("{}", "[!] macOS 需要本地编译 plugin-loader，请确保 Go 已安装。".yellow());
        return false;
    }

    // Linux / Windows：从 OBS 下载预编译二进制
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(parent) = plugin_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let url = if cfg!(target_os = "windows") {
            "https://cloud-sumi-use.obs.cn-east-3.myhuaweicloud.com/plugin-loader.exe"
        } else {
            "https://cloud-sumi-use.obs.cn-east-3.myhuaweicloud.com/plugin-loader"
        };

        if !download_file(url, &plugin_path) {
            return false;
        }

        // 在 Linux 上显式设置可执行权限，避免 ARMHF 等平台出现 Permission Denied
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&plugin_path, fs::Permissions::from_mode(0o755));
        }

        true
    }
}

pub fn ensure_plugins_downloaded() -> bool {
    let plugins_dir = plugin_dir();
    let _ = fs::create_dir_all(&plugins_dir);
    let stlink_dir = plugins_dir.join("stlink_v2");
    let cmsis_dir = plugins_dir.join("cmsis_dap");
    let mut ok = true;
    if !stlink_dir.exists() {
        let url = "https://cloud-sumi-use.obs.cn-east-3.myhuaweicloud.com/stlink_v2.zip";
        if !download_and_extract_zip(url, &plugins_dir) {
            log_warn("Failed to download stlink_v2");
            ok = false;
        }
    }
    if !cmsis_dir.exists() {
        let url = "https://cloud-sumi-use.obs.cn-east-3.myhuaweicloud.com/cmsis_dap.zip";
        if !download_and_extract_zip(url, &plugins_dir) {
            log_warn("Failed to download cmsis_dap");
            ok = false;
        }
    }
    ok
}


/// 创建 Go 安装脚本，提供安装方案给用户。
pub fn create_go_install_script() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let script_path = if cfg!(target_os = "windows") {
        cwd.join("install_go.ps1")
    } else {
        cwd.join("install_go.sh")
    };

    let content = if cfg!(target_os = "windows") {
        r#"Write-Host '安装 Go 运行时环境'
if (Get-Command winget -ErrorAction SilentlyContinue) {
    winget install --id GoLang.Go -e
} else {
    Write-Host '请手动从 https://go.dev/dl/ 下载并安装 Go。'
}
"#
    } else if cfg!(target_os = "macos") {
        r#"#!/bin/sh
set -e
if command -v brew >/dev/null 2>&1; then
    brew install go
else
    echo '请安装 Homebrew (https://brew.sh) 后重试，或手动从 https://go.dev/dl/ 下载 Go。'
fi
"#
    } else {
        r#"#!/bin/sh
set -e
if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update && sudo apt-get install -y golang-go
elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y golang
elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S --noconfirm go
elif command -v zypper >/dev/null 2>&1; then
    sudo zypper install -y golang
else
    echo '请手动从 https://go.dev/dl/ 下载并安装 Go。'
fi
"#
    };

    if fs::write(&script_path, content).is_ok() {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755));
        }
        Some(script_path)
    } else {
        None
    }
}

/// 准备运行时环境，尝试自动下载 plugin-loader 源码并创建 Go 安装脚本。
pub fn prepare_runtime_environment() -> bool {
    let mut ok = true;
    let logs_dir = logs_dir();
    let _ = fs::create_dir_all(&logs_dir);
    let go_installed = check_go_installed();
    let loader_source_ready = ensure_plugin_loader_source();

    if !go_installed {
        log_warn("Go 编译环境未检测到。");
        if let Some(script) = create_go_install_script() {
            log_info(&format!("Go 安装脚本已创建: {}", script.display()));
            println!("{}", format!("[!] Go 安装脚本已创建: {}", script.display()).yellow());
        } else {
            println!("{}", "[!] 无法创建 Go 安装脚本，请手动安装 Go。".yellow());
        }
        ok = false;
    }

    if !loader_source_ready {
        log_warn("plugin-loader 源码环境未准备好。");
        if let Some(repo) = Some(select_plugin_loader_repo()) {
            println!("{}", format!("[!] plugin-loader 源码未找到，已尝试从 {} 获取。", repo).yellow());
        }
        ok = false;
    }

    if !ensure_plugins_downloaded() {
        log_warn("插件下载失败。");
        ok = false;
    }

    // 只要 Go 可用就尝试构建 plugin-loader（macOS 无预编译二进制，强制走此路径；
    // Linux 上下载失败时也可作为 fallback）
    if go_installed {
        if !ensure_plugin_loader_binary() {
            log_warn("plugin-loader 二进制构建失败。");
            ok = false;
        }
    }
    ok
}

pub fn find_plugin_loader_tool() -> Option<String> {
    if let Ok(custom) = env::var("PLUGIN_LOADER_BIN") {
        if Path::new(&custom).is_file() {
            return Some(custom);
        }
    }

    let mut paths = Vec::new();
    if let Some(root) = find_project_root() {
        let root_plugins = root.join("plugins").join(plugin_loader_executable_name());
        let root_loader = root.join("plugin-loader").join(plugin_loader_executable_name());
        paths.push(root_plugins);
        paths.push(root_loader);
    }
    paths.push(PathBuf::from("plugins").join(plugin_loader_executable_name()));
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

    let build_ok = match build_result {
        Ok(status) if status.success() => output_file.is_file(),
        _ => false,
    };

    if !build_ok {
        return false;
    }

    // 在某些文件系统（tmpfs、网络挂载等）下 go build 输出的二进制可能缺少可执行位
    #[cfg(unix)]
    {
        let _ = fs::set_permissions(&output_file, fs::Permissions::from_mode(0o755));
    }

    true
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
