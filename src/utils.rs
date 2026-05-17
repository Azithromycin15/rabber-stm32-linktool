//! # 工具函数模块
//!
//! 命令执行、工具查找、权限检查、环境准备等通用功能。

use colored::Colorize;
use crate::logger::warn as log_warn;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// 命令执行结果
pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
}

/// 检查是否以 root 身份运行
pub fn is_root() -> bool {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        Command::new("id").arg("-u")
            .output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("net").args(["session"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// 构建需要特权的命令: 非 root 时自动用 sudo 包装。
/// 设置环境变量 `RABBER_NO_SUDO=1` 可禁用自动 sudo。
pub fn build_privileged_command(program: &str) -> Command {
    if is_root() || env::var("RABBER_NO_SUDO").is_ok() {
        Command::new(program)
    } else {
        let mut c = Command::new("sudo");
        c.arg(program);
        c
    }
}

/// 执行外部命令
pub fn execute_command(cmd: &str, args: &[&str]) -> CommandResult {
    match Command::new(cmd).args(args)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()
    {
        Ok(o) => CommandResult {
            status: o.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        },
        Err(_) => CommandResult { status: -1, stdout: String::new() },
    }
}

/// 在候选路径和 PATH 中查找工具
pub fn find_tool(name: &str, candidates: &[&str]) -> Option<String> {
    candidates.iter().find(|p| Path::new(p).is_file())
        .map(|s| s.to_string())
        .or_else(|| which::which(name).ok().map(|p| p.to_string_lossy().into_owned()))
}

// ── ST-Link 工具查找 ──

pub fn find_stlink_cli_tool() -> Option<String> {
    let (name, paths) = if cfg!(target_os = "windows") {
        ("ST-LINK_CLI.exe", STLINK_WIN_PATHS)
    } else {
        ("st-info", STLINK_NIX_CLI_PATHS)
    };
    find_tool(name, &paths)
}

#[allow(dead_code)]
pub fn find_stlink_programmer_tool() -> Option<String> {
    let (name, paths) = if cfg!(target_os = "windows") {
        ("ST-LINK_CLI.exe", STLINK_WIN_PATHS)
    } else {
        ("st-flash", STLINK_NIX_FLASH_PATHS)
    };
    find_tool(name, &paths)
}

const STLINK_WIN_PATHS: &[&str] = &[
    "C:\\Program Files (x86)\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
    "C:\\Program Files\\STMicroelectronics\\STM32 ST-LINK Utility\\ST-LINK_CLI.exe",
    "ST-LINK_CLI.exe",
];

const STLINK_NIX_CLI_PATHS: &[&str] = &[
    "/usr/bin/st-info", "/usr/local/bin/st-info", "/bin/st-info",
    "/usr/bin/stlink-info", "/usr/local/bin/stlink-info",
];

#[allow(dead_code)]
const STLINK_NIX_FLASH_PATHS: &[&str] = &[
    "/usr/bin/st-flash", "/usr/local/bin/st-flash", "/bin/st-flash",
    "/usr/bin/stlink-flash", "/usr/local/bin/stlink-flash",
];

// ── 路径查询 ──

/// 返回当前可执行文件所在目录
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))
}

/// 查找项目根目录。
///
/// 优先级:
/// 1. PROJECT_ROOT 环境变量
/// 2. 从当前目录向上查找 Cargo.toml + plugins (开发环境)
/// 3. 从可执行文件目录向上查找 plugins + plugin-loader (发布环境)
pub fn find_project_root() -> Option<PathBuf> {
    if let Ok(root) = env::var("PROJECT_ROOT") {
        let p = PathBuf::from(&root);
        if p.is_dir() { return Some(p); }
    }
    // 开发环境: 从当前目录向上查找
    let mut path = env::current_dir().ok()?;
    loop {
        if path.join("Cargo.toml").is_file() && path.join("plugins").is_dir() {
            return Some(path);
        }
        if !path.pop() { break; }
    }
    // 发布环境: 从可执行文件目录向上查找
    if let Some(mut exe_path) = exe_dir() {
        loop {
            if exe_path.join("plugins").is_dir() {
                return Some(exe_path);
            }
            if !exe_path.pop() { break; }
        }
    }
    None
}

pub fn plugin_dir() -> PathBuf {
    env::var("PLUGIN_DIR").ok().map(PathBuf::from)
        .or_else(|| find_project_root().map(|r| r.join("plugins")))
        .or_else(|| exe_dir().map(|d| d.join("plugins")).filter(|p| p.is_dir()))
        .unwrap_or_else(|| PathBuf::from("plugins"))
}

pub fn logs_dir() -> PathBuf {
    env::var("LOGS_DIR").ok().map(PathBuf::from)
        .or_else(|| find_project_root().map(|r| r.join("logs")))
        .or_else(|| exe_dir().map(|d| d.join("logs")).filter(|p| p.is_dir()))
        .unwrap_or_else(|| PathBuf::from("logs"))
}

pub fn manifest_path() -> PathBuf {
    env::var("PLUGIN_MANIFEST").ok().map(PathBuf::from)
        .unwrap_or_else(|| plugin_dir().join("manifest.yaml"))
}

pub fn plugin_loader_dir() -> PathBuf {
    env::var("PLUGIN_LOADER_DIR").ok().map(PathBuf::from)
        .or_else(|| find_project_root().map(|r| r.join("plugin-loader")))
        .or_else(|| exe_dir().map(|d| d.join("plugin-loader")).filter(|p| p.is_dir()))
        .unwrap_or_else(|| PathBuf::from("plugin-loader"))
}

/// 返回编译时嵌入的 Cargo.toml 版本号
pub fn cargo_package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ── 依赖检测 ──

pub fn check_openocd_installed() -> bool {
    find_tool("openocd", &["openocd", "/usr/bin/openocd", "/usr/local/bin/openocd"]).is_some()
}

pub fn check_stlink_tools_installed() -> bool {
    find_stlink_cli_tool().is_some()
}

fn check_go_installed() -> bool {
    which::which("go").is_ok()
}

// ── 插件加载器 ──

fn plugin_loader_exe() -> &'static str {
    if cfg!(target_os = "windows") { "plugin-loader.exe" } else { "plugin-loader" }
}

pub fn find_plugin_loader_tool() -> Option<String> {
    if let Ok(custom) = env::var("PLUGIN_LOADER_BIN") {
        if Path::new(&custom).is_file() { return Some(custom); }
    }
    let exe = plugin_loader_exe();
    let mut try_paths: Vec<PathBuf> = Vec::new();
    if let Some(root) = find_project_root() {
        try_paths.push(root.join("plugins").join(exe));
        try_paths.push(root.join("plugin-loader").join(exe));
    }
    // 可执行文件相邻目录
    if let Some(exe_dir) = exe_dir() {
        try_paths.push(exe_dir.join(exe));
        try_paths.push(exe_dir.join("plugins").join(exe));
    }
    try_paths.push(PathBuf::from("plugins").join(exe));
    try_paths.push(PathBuf::from("plugin-loader").join(exe));
    try_paths.push(PathBuf::from("/usr/local/bin").join(exe));
    try_paths.push(PathBuf::from("/usr/bin").join(exe));
    try_paths.iter().find(|p| p.is_file()).map(|p| p.to_string_lossy().into_owned())
}

pub fn ensure_plugin_loader_binary() -> bool {
    if find_plugin_loader_tool().is_some() { return true; }

    let dir = plugin_loader_dir();
    let out = dir.join(plugin_loader_exe());

    // 尝试从项目根目录复制预编译的二进制
    if let Some(root) = find_project_root() {
        for src_dir in &["plugins", "release"] {
            let prebuilt = root.join(src_dir).join(plugin_loader_exe());
            if prebuilt.is_file() {
                let _ = fs::create_dir_all(&dir);
                if fs::copy(&prebuilt, &out).is_ok() {
                    println!("{}", format!("[✓] plugin-loader 已复制 (来自 {}): {}", src_dir, out.display()).green());
                    return true;
                }
            }
        }
    }

    // 尝试 Go 编译
    let ok = Command::new("go").args(["build", "-o"]).arg(&out)
        .current_dir(&dir).status()
        .map(|s| s.success() && out.is_file()).unwrap_or(false);
    if ok {
        println!("{}", format!("[✓] plugin-loader 构建成功: {}", out.display()).green());
    }
    ok
}

// ── 插件源码环境 ──

const GITEE_REPO: &str = "https://gitee.com/kroazithromycin/rabber-stm32-linktool-plugin-loader.git";
const GITHUB_REPO: &str = "https://github.com/Azithromycin15/rabber-stm32-linktool-plugin-loader.git";

fn select_loader_repo() -> &'static str {
    // 尝试通过 ipinfo.io 判断地理位置，优先使用国内镜像
    for fetcher in &["curl", "wget"] {
        if which::which(fetcher).is_err() { continue; }
        let url = "https://ipinfo.io/country";
        let args: Vec<&str> = if *fetcher == "curl" { vec!["-s", url] } else { vec!["-qO-", url] };
        if let Ok(o) = Command::new(fetcher).args(&args).output() {
            if o.status.success() && String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("CN") {
                return GITEE_REPO;
            }
        }
    }
    GITHUB_REPO
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn ensure_plugin_loader_source() -> bool {
    let dir = plugin_loader_dir();
    if dir.join("main.go").is_file() { return true; }

    // 首选：从远程仓库 git clone
    if which::which("git").is_ok() {
        let parent = dir.parent().unwrap_or(Path::new("."));
        let _ = fs::create_dir_all(parent);
        let ok = Command::new("git").args(["clone", "--depth", "1", select_loader_repo()])
            .arg(&dir).status().map(|s| s.success()).unwrap_or(false);
        if ok {
            println!("{}", format!("[✓] plugin-loader 源码已克隆: {}", dir.display()).green());
            return true;
        }
        log_warn("git clone plugin-loader 失败, 尝试本地回退...");
    }

    // 回退：从项目根目录复制本地源码
    if let Some(root) = find_project_root() {
        let src = root.join("plugin-loader");
        if src.join("main.go").is_file() {
            let _ = fs::create_dir_all(dir.parent().unwrap_or(Path::new(".")));
            if copy_dir_all(&src, &dir).is_ok() && dir.join("main.go").is_file() {
                println!("{}", format!("[✓] plugin-loader 源码已复制 (本地回退): {}", dir.display()).green());
                return true;
            }
        }
    }

    false
}

fn ensure_plugins_downloaded() -> bool {
    let dir = plugin_dir();
    dir.join("manifest.yaml").is_file() || dir.join("stlink_v2").is_dir() || true
}

fn create_go_install_script() -> Option<PathBuf> {
    let path = logs_dir().join("install_go.sh");
    let script = r##"#!/bin/bash
set -e
echo "Installing Go..."
if command -v apt-get &>/dev/null; then sudo apt-get update && sudo apt-get install -y golang-go
elif command -v pacman &>/dev/null; then sudo pacman -S --noconfirm go
elif command -v dnf &>/dev/null; then sudo dnf install -y golang
elif command -v brew &>/dev/null; then brew install go
else echo "Cannot auto-install Go. Visit https://go.dev/dl/"; exit 1; fi
echo "Go installed"
"##;
    fs::write(&path, script).ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(m) = fs::metadata(&path) {
            let mut p = m.permissions();
            p.set_mode(0o755);
            let _ = fs::set_permissions(&path, p);
        }
    }
    Some(path)
}

// ── 运行时环境 ──

pub fn prepare_runtime_environment() -> bool {
    let mut ok = true;
    let _ = fs::create_dir_all(&logs_dir());
    if !check_go_installed() {
        log_warn("Go 未检测到 (将尝试使用预编译 plugin-loader)");
        if let Some(s) = create_go_install_script() {
            println!("{}", format!("[!] Go 安装脚本已创建: {}", s.display()).yellow());
        }
    }
    if !ensure_plugin_loader_source() {
        log_warn("plugin-loader 源码未准备好");
        ok = false;
    }
    if !ensure_plugins_downloaded() { log_warn("插件下载失败"); ok = false; }
    // 始终尝试获取/构建 plugin-loader（支持预编译二进制回退）
    if !ensure_plugin_loader_binary() {
        log_warn("plugin-loader 不可用");
        ok = false;
    }
    ok
}

pub fn is_project_root() -> bool {
    Path::new("Cargo.toml").is_file() && Path::new("plugins").is_dir()
}

pub fn print_environment_summary() {
    println!("{}", "[环境摘要]".cyan());
    println!("  OS:          {}", std::env::consts::OS);
    if let Some(r) = find_project_root() { println!("  Project:     {}", r.display()); }
    println!("  Go:          {}", if check_go_installed() { "✓" } else { "✗" });
    println!("  ST-Link:     {}", if check_stlink_tools_installed() { "✓" } else { "✗" });
    println!("  OpenOCD:     {}", if check_openocd_installed() { "✓" } else { "✗" });
    println!("  Root:        {}", if is_root() { "yes" } else { "no" });
}
