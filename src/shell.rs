//! # 交互式 Shell
//!
//! 命令行交互界面，内置命令 + 插件命令。

use colored::*;
use std::env;
use std::path::{Path, PathBuf};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::output::show_help;
use crate::plugin::{ComponentInfo, PluginManager};
use crate::stlink::get_mcu_info_via_swd;
use crate::utils::{build_privileged_command, find_plugin_loader_tool, manifest_path, plugin_dir};

pub fn interactive_mode(plugin_manager: &mut Option<PluginManager>, default_downloader: Option<String>) {
    let mut rl = Editor::<(), _>::new().expect("无法初始化编辑器");
    let mut cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

    loop {
        match rl.readline(&format!("rabber:{}> ", cwd.display())) {
            Ok(line) => {
                let t = line.trim();
                if t.is_empty() { continue; }
                rl.add_history_entry(t).ok();
                if let Some(d) = dispatch(t, plugin_manager, default_downloader.as_deref(), &mut cwd) {
                    let _ = env::set_var("OLDPWD", cwd.to_string_lossy().as_ref());
                    match env::set_current_dir(&d) {
                        Ok(()) => { cwd = d; crate::logger::info(&format!("cd → {}", cwd.display())); }
                        Err(e) => println!("{}", format!("cd 失败: {}", e).red()),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => { println!("^C"); break; }
            Err(ReadlineError::Eof) => break,
            Err(e) => { println!("读取错误: {}", e); break; }
        }
    }
}

fn dispatch(line: &str, mgr: &mut Option<PluginManager>, dl: Option<&str>, cwd: &mut PathBuf) -> Option<PathBuf> {
    let mut parts = line.split_whitespace();
    let cmd = parts.next()?;

    match cmd {
        "exit" | "quit" => { println!("退出。"); std::process::exit(0); }
        "help" => if let Some(pid) = parts.next() {
            if pid == "plugin" {
                mgr.as_ref().map(|m| m.help_all_plugins()).unwrap_or_else(|| println!("{}", "未加载插件清单。".yellow()));
            } else {
                mgr.as_ref().map(|m| m.help(pid)).unwrap_or_else(|| println!("{}", "未加载插件清单。".yellow()));
            }
        } else { show_help(); }
        "pwd" => println!("{}", cwd.display()),
        "ls" => {
            let mut cmd = std::process::Command::new("ls");
            for arg in parts { cmd.arg(arg); }
            match cmd.status() {
                Ok(s) => {
                    if !s.success() {
                        println!("{}", format!("ls 退出码: {}", s.code().unwrap_or(-1)).red());
                    }
                }
                Err(e) => println!("{}", format!("执行 ls 失败: {}", e).red()),
            }
        }
        "cd" => return cd(parts, cwd),
        "info" => {
            let info = get_mcu_info_via_swd();
            if !info.chip_id.is_empty() { crate::output::print_mcu_info(&info); }
            else { println!("{}", "无法获取 MCU 信息。".red()); }
        }
        "flash" => flash(parts, mgr.as_ref(), dl, cwd),
        "reset" => reset(mgr.as_ref(), dl, cwd),
        pid => {
            // plugin 命令: plugin list|discover|refresh|help
            if pid == "plugin" {
                match parts.next() {
                    Some("discover") | Some("-d") => {
                        let mp = manifest_path();
                        match PluginManager::load_from(&mp.to_string_lossy()) {
                            Some(new_mgr) => {
                                *mgr = Some(new_mgr);
                                let count = mgr.as_ref().map(|m| m.count()).unwrap_or(0);
                                println!("{}", format!("插件列表已热加载 (discover), {} 个组件", count).green());
                                mgr.as_ref().map(|m| m.list());
                            }
                            None => println!("{}", "无法加载 manifest.yaml".red()),
                        }
                    }
                    Some("refresh") | Some("-r") => {
                        let pd = plugin_dir();
                        let mp = manifest_path();
                        match PluginManager::probe_and_generate_manifest(&pd, &mp) {
                            Some(new_mgr) => {
                                *mgr = Some(new_mgr);
                                let count = mgr.as_ref().map(|m| m.count()).unwrap_or(0);
                                println!("{}", format!("插件已重新探测并刷新 (refresh), {} 个组件", count).green());
                                mgr.as_ref().map(|m| m.list());
                            }
                            None => println!("{}", "重新探测插件失败".red()),
                        }
                    }
                    Some("list") | Some("-l") | Some("help") => {
                        mgr.as_ref().map(|m| m.help_all_plugins()).unwrap_or_else(|| println!("{}", "未加载插件清单。".yellow()));
                    }
                    _ => {
                        println!("{}", "用法: plugin list|discover|refresh|help".yellow());
                    }
                }
                return None;
            }
            // 插件命令（按 manifest 中的 command 字段匹配）
            if let Some(m) = mgr.as_ref() {
                if let Some(c) = m.find_by_command(pid) {
                    let cid = &c.id;
                    if let Some(act) = parts.next() {
                        if act == "help" { m.help(cid); return None; }
                        if !m.has_action(cid, act) {
                            println!("{}", format!("插件 '{}' 不支持 '{}'", pid, act).red());
                            m.help(cid);
                            return None;
                        }
                        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
                        return run_plugin(c, act, &args, cwd);
                    } else {
                        println!("{}", "用法: <插件ID> <命令> [选项]".yellow());
                        m.help(cid);
                    }
                } else {
                    println!("{}: {}", "未知命令".red(), cmd);
                    println!("输入 'help' 查看可用命令。");
                }
            }
        }
    }
    None
}

fn cd(mut parts: std::str::SplitWhitespace, cwd: &PathBuf) -> Option<PathBuf> {
    let target = match parts.next() {
        None => env::var("HOME").ok().map(PathBuf::from)?,
        Some("~") => env::var("HOME").ok().map(PathBuf::from)?,
        Some("-") => env::var("OLDPWD").ok().map(PathBuf::from)?,
        Some(p) if p == ".." || p.starts_with("../") || p.starts_with("./") || p.starts_with('/') => PathBuf::from(p),
        Some(p) => cwd.join(p),
    };
    match target.canonicalize() {
        Ok(p) => Some(p),
        Err(_) => { println!("{}", format!("目录不存在: {}", target.display()).red()); None }
    }
}

fn flash(mut parts: std::str::SplitWhitespace, mgr: Option<&PluginManager>, dl: Option<&str>, cwd: &Path) {
    let file = match parts.next() { Some(f) => f, None => { println!("{}", "用法: flash <file>".red()); return; } };
    let m = match mgr { Some(m) => m, None => { println!("{}", "插件管理器不可用。".red()); return; } };
    let c = dl.and_then(|id| m.find(id)).or_else(|| m.default_downloader());
    match c { Some(c) => { run_plugin(c, "flash", &[file.into()], cwd); } None => println!("{}", "未找到下载器。".red()) }
}

fn reset(mgr: Option<&PluginManager>, dl: Option<&str>, cwd: &Path) {
    let m = match mgr { Some(m) => m, None => { println!("{}", "插件管理器不可用。".red()); return; } };
    let c = dl.and_then(|id| m.find(id)).or_else(|| m.default_downloader());
    match c { Some(c) => { run_plugin(c, "reset", &[], cwd); } None => println!("{}", "未找到下载器。".red()) }
}

/// 需要将首个参数作为文件路径传给 `--file` 的 action 名称
const FILE_ACTION_NAMES: &[&str] = &["flash", "verify", "compile", "strip"];

/// Execute a plugin action via plugin-loader and return an optional cd target path.
/// When the Python component outputs `__RABBER_CD__:<path>`, that path is returned
/// so the interactive shell can automatically change directory.
fn run_plugin(component: &ComponentInfo, action: &str, args: &[String], cwd: &Path) -> Option<PathBuf> {
    let loader = match find_plugin_loader_tool() {
        Some(p) => p,
        None => { println!("{}", "plugin-loader 未找到".red()); return None; }
    };
    let mut cmd = build_privileged_command(&loader);
    cmd.arg("--manifest").arg(manifest_path().to_string_lossy().as_ref())
        .arg("--component").arg(&component.id).arg("--action").arg(action)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if FILE_ACTION_NAMES.contains(&action) {
        if let Some(f) = args.first() {
            let resolved = if Path::new(f).is_relative() {
                cwd.join(f)
            } else {
                PathBuf::from(f)
            };
            cmd.arg("--file").arg(resolved.to_string_lossy().as_ref());
            if args.len() > 1 {
                cmd.arg("--");
                for a in &args[1..] { cmd.arg(a); }
            }
        } else {
            println!("{}", format!("{} 需要文件路径", action).red());
            return None;
        }
    } else if !args.is_empty() {
        cmd.arg("--");
        for a in args { cmd.arg(a); }
    }

    println!("{}", format!("执行 {} {}...", component.id, action).cyan());
    match cmd.output() {
        Ok(output) => {
            // Print stderr first (diagnostics)
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                eprint!("{}", stderr);
            }
            // Print stdout (scanning for cd marker)
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut cd_target: Option<PathBuf> = None;
            for line in stdout.lines() {
                if let Some(path_str) = line.strip_prefix("__RABBER_CD__:") {
                    cd_target = Some(PathBuf::from(path_str.trim()));
                } else {
                    println!("{}", line);
                }
            }
            if output.status.success() {
                println!("{}", "成功".green());
            } else {
                println!("{}", "失败".red());
            }
            cd_target
        }
        Err(e) => {
            println!("{}", format!("错误: {}", e).red());
            None
        }
    }
}
