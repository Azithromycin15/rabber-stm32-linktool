//! # 交互式 Shell
//!
//! 命令行交互界面，内置命令 + 插件命令。
//! 支持 Tab 补全、智能提示、历史搜索 (Ctrl-R)、路径缩写。

use colored::*;
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationResult, ValidationContext};
use rustyline::{Context, Editor, Helper};

use crate::output::{show_help, show_command_help};
use crate::plugin::{ComponentInfo, PluginManager};
use crate::stlink::get_mcu_info_via_swd;
use crate::t;
use crate::tfmt;
use crate::utils::{build_privileged_command, find_plugin_loader_tool, manifest_path, plugin_dir};

// ── Shell Helper: Tab 补全 + 路径缩写 + 输入校验 ──

struct ShellHelper {
    builtins: Vec<String>,
    plugin_commands: Vec<String>,
    cwd: PathBuf,
}

impl ShellHelper {
    fn new(mgr: Option<&PluginManager>, cwd: &PathBuf) -> Self {
        let builtins = vec![
            "help", "?", "pwd", "cd", "ls", "dir", "info", "flash", "reset",
            "exit", "quit", "clear", "plugin", "version",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let plugin_commands = mgr
            .map(|m| m.all_commands())
            .unwrap_or_default();

        ShellHelper { builtins, plugin_commands, cwd: cwd.clone() }
    }

    fn all_commands(&self) -> Vec<&str> {
        self.builtins.iter().map(|s| s.as_str())
            .chain(self.plugin_commands.iter().map(|s| s.as_str()))
            .collect()
    }
}

// ── Completer ──

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (word_start, word) = extract_current_word(line, pos);
        let prior = line[..word_start].trim();
        let is_first_word = prior.is_empty();

        if is_first_word {
            let candidates: Vec<Pair> = self.all_commands()
                .iter()
                .filter(|c| c.starts_with(word))
                .map(|c| Pair { display: c.to_string(), replacement: c.to_string() })
                .collect();
            return Ok((word_start, candidates));
        }

        let first_word = line.split_whitespace().next().unwrap_or("");

        match first_word {
            "cd" => return complete_dirs(line, pos),
            "flash" | "ls" | "dir" => return complete_files(line, pos),
            "help" | "?" => {
                let words: Vec<&str> = line[..pos].split_whitespace().collect();
                if words.len() == 2 {
                    let candidates: Vec<Pair> = self.all_commands()
                        .iter()
                        .filter(|c| c.starts_with(word))
                        .map(|c| Pair { display: c.to_string(), replacement: c.to_string() })
                        .collect();
                    return Ok((word_start, candidates));
                }
            }
            "plugin" => {
                let words: Vec<&str> = line[..pos].split_whitespace().collect();
                if words.len() == 2 {
                    let subs = vec!["list", "discover", "refresh", "help"];
                    let candidates: Vec<Pair> = subs
                        .iter()
                        .filter(|s| s.starts_with(word))
                        .map(|s| Pair { display: s.to_string(), replacement: s.to_string() })
                        .collect();
                    return Ok((word_start, candidates));
                }
            }
            _ => {}
        }

        Ok((pos, vec![]))
    }
}

fn complete_dirs(line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (start, word) = extract_current_word(line, pos);
    let candidates = glob_paths(word, true);
    Ok((start, candidates))
}

fn complete_files(line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (start, word) = extract_current_word(line, pos);
    let candidates = glob_paths(word, false);
    Ok((start, candidates))
}

/// 简单的路径补全：列出匹配前缀的文件/目录
fn glob_paths(word: &str, dirs_only: bool) -> Vec<Pair> {
    let path = Path::new(word);
    let (base_dir, prefix) = if word.is_empty() {
        (Path::new("."), "")
    } else if word.ends_with('/') {
        (Path::new(word), "")
    } else {
        match path.parent() {
            Some(p) if p.as_os_str().is_empty() => (Path::new("."), path.file_name().unwrap_or_default().to_str().unwrap_or("")),
            Some(p) => (p, path.file_name().unwrap_or_default().to_str().unwrap_or("")),
            None => (Path::new("."), word),
        }
    };

    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(prefix) && !name_str.starts_with('.') {
                let is_dir = entry.path().is_dir();
                if dirs_only && !is_dir {
                    continue;
                }
                let display = if is_dir { format!("{}/", name_str) } else { name_str.to_string() };
                let replacement = if word.is_empty() || word.ends_with('/') {
                    format!("{}{}", word, display)
                } else if let Some(parent) = path.parent() {
                    if parent.as_os_str().is_empty() {
                        display.clone()
                    } else {
                        format!("{}/{}", parent.display(), display)
                    }
                } else {
                    display.clone()
                };

                if seen.insert(replacement.clone()) {
                    candidates.push(Pair { display, replacement });
                }
            }
        }
    }
    candidates
}

fn extract_current_word(line: &str, pos: usize) -> (usize, &str) {
    let bytes = line.as_bytes();
    let clamped = pos.min(line.len());
    let mut start = clamped;
    while start > 0 && bytes[start - 1] != b' ' {
        start -= 1;
    }
    (start, &line[start..clamped])
}

// ── Hinter ──

impl Hinter for ShellHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

// ── Highlighter ──

impl Highlighter for ShellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        false
    }
}

// ── Validator ──

impl Validator for ShellHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellHelper {}

// ── 公共接口 ──

pub fn interactive_mode(plugin_manager: &mut Option<PluginManager>, default_downloader: Option<String>) {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let helper = ShellHelper::new(plugin_manager.as_ref(), &cwd);
    let mut rl = Editor::new().expect(t!("无法初始化编辑器", "Failed to initialize editor"));
    rl.set_helper(Some(helper));
    let _ = rl.load_history("rabber_history.txt");

    let mut cwd = cwd;

    loop {
        let prompt = format_prompt(&cwd);
        match rl.readline(&prompt) {
            Ok(line) => {
                let t = line.trim();
                if t.is_empty() { continue; }
                rl.add_history_entry(t).ok();
                if let Some(d) = dispatch(t, plugin_manager, default_downloader.as_deref(), &mut cwd) {
                    let _ = env::set_var("OLDPWD", cwd.to_string_lossy().as_ref());
                    match env::set_current_dir(&d) {
                        Ok(()) => {
                            cwd = d;
                            crate::logger::info(&format!("cd → {}", cwd.display()));
                            if let Some(h) = rl.helper_mut() {
                                h.cwd = cwd.clone();
                            }
                        }
                        Err(e) => println!("{}", format!("{}: {}", t!("cd 失败", "cd failed"), e).red()),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", t!("^C (再按一次 Ctrl-C 或输入 exit 退出)", "^C (press again or type exit to quit)"));
            }
            Err(ReadlineError::Eof) => {
                println!("{}", t!("退出。", "Goodbye."));
                break;
            }
            Err(e) => {
                println!("{}: {}", t!("读取错误", "Read error"), e);
                break;
            }
        }
    }
    let _ = rl.save_history("rabber_history.txt");
}

fn format_prompt(cwd: &Path) -> String {
    let home = env::var("HOME").ok().map(PathBuf::from);
    let display = match home {
        Some(ref h) if cwd.starts_with(h) => {
            if cwd == *h { "~".to_string() }
            else { format!("~/{}", cwd.strip_prefix(h).unwrap().display()) }
        }
        _ => cwd.display().to_string(),
    };

    let short = if display.len() > 50 && !display.starts_with('/') {
        let parts: Vec<&str> = display.split('/').collect();
        if parts.len() > 3 {
            format!(".../{}", parts[parts.len()-2..].join("/"))
        } else { display }
    } else { display };

    format!("{} {} ", short.cyan().bold(), "❯".white().bold())
}

// ── 命令分发 ──

fn dispatch(line: &str, mgr: &mut Option<PluginManager>, dl: Option<&str>, cwd: &mut PathBuf) -> Option<PathBuf> {
    let mut parts = line.split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c,
        None => return None,
    };

    match cmd {
        "exit" | "quit" => {
            println!("{}", t!("再见 👋", "Goodbye 👋"));
            std::process::exit(0);
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H");
            return None;
        }
        "version" => {
            println!("rabber {}", env!("CARGO_PKG_VERSION"));
            return None;
        }
        "help" | "?" => {
            if let Some(arg) = parts.next() {
                if arg == "plugin" {
                    mgr.as_ref()
                        .map(|m| m.help_all_plugins())
                        .unwrap_or_else(|| println!("{}", t!("未加载插件清单。", "Plugin manifest not loaded.").yellow()));
                } else {
                    show_command_help(arg, mgr.as_ref());
                }
            } else {
                show_help(mgr.as_ref());
            }
            return None;
        }
        "pwd" => {
            println!("{}", cwd.display());
            return None;
        }
        "ls" | "dir" => {
            let mut child = match std::process::Command::new("ls")
                .args(parts)
                .current_dir(cwd)
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    println!("{}", format!("{}: {}", t!("执行 ls 失败", "ls failed"), e).red());
                    return None;
                }
            };
            let _ = child.wait();
            return None;
        }
        "cd" => return cd(parts, cwd),
        "info" => {
            let info = get_mcu_info_via_swd();
            if !info.chip_id.is_empty() {
                crate::output::print_mcu_info(&info);
            } else {
                println!("{}", t!("无法获取 MCU 信息。请确认 ST-Link 已连接。", "Cannot get MCU info. Please confirm ST-Link is connected.").red());
            }
            return None;
        }
        "flash" => {
            flash(parts, mgr.as_ref(), dl, cwd);
            return None;
        }
        "reset" => {
            reset(mgr.as_ref(), dl, cwd);
            return None;
        }
        "plugin" => {
            match parts.next() {
                Some("discover") | Some("-d") => {
                    let mp = manifest_path();
                    match PluginManager::load_from(&mp.to_string_lossy()) {
                        Some(new_mgr) => {
                            *mgr = Some(new_mgr);
                            let count = mgr.as_ref().map(|m| m.count()).unwrap_or(0);
                            println!("{}", tfmt!("插件列表已热加载 (discover), {} 个组件", "Plugin list hot-reloaded (discover), {} components", count).green());
                            mgr.as_ref().map(|m| m.list());
                        }
                        None => println!("{}", t!("无法加载 manifest.yaml", "Cannot load manifest.yaml").red()),
                    }
                }
                Some("refresh") | Some("-r") => {
                    let pd = plugin_dir();
                    let mp = manifest_path();
                    match PluginManager::probe_and_generate_manifest(&pd, &mp) {
                        Some(new_mgr) => {
                            *mgr = Some(new_mgr);
                            let count = mgr.as_ref().map(|m| m.count()).unwrap_or(0);
                            println!("{}", tfmt!("插件已重新探测并刷新 (refresh), {} 个组件", "Plugins re-probed and refreshed (refresh), {} components", count).green());
                            mgr.as_ref().map(|m| m.list());
                        }
                        None => println!("{}", t!("重新探测插件失败", "Re-probe plugins failed").red()),
                    }
                }
                Some("list") | Some("-l") | Some("help") => {
                    mgr.as_ref()
                        .map(|m| m.help_all_plugins())
                        .unwrap_or_else(|| println!("{}", t!("未加载插件清单。", "Plugin manifest not loaded.").yellow()));
                }
                _ => {
                    println!("{}", t!("用法: plugin list|discover|refresh|help", "Usage: plugin list|discover|refresh|help").yellow());
                }
            }
            return None;
        }
        pid => {
            if let Some(m) = mgr.as_ref() {
                if let Some(c) = m.find_by_command(pid) {
                    let cid = &c.id;
                    if let Some(act) = parts.next() {
                        if act == "help" {
                            m.help(cid);
                            return None;
                        }
                        if !m.has_action(cid, act) {
                            println!("{}", tfmt!("插件 '{}' 不支持 '{}'", "Plugin '{}' does not support '{}'", pid, act).red());
                            m.help(cid);
                            return None;
                        }
                        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
                        return run_plugin(c, act, &args, cwd);
                    } else {
                        println!("{}", t!("用法: <插件ID> <命令> [选项]", "Usage: <pluginID> <command> [options]").yellow());
                        m.help(cid);
                    }
                } else {
                    suggest_command(cmd, mgr.as_ref());
                }
            } else {
                println!("{}: {}", t!("未知命令", "Unknown command").red(), cmd);
                println!("{} 'help' {}。", t!("输入", "Type"), t!("查看可用命令", "to see available commands"));
            }
        }
    }
    None
}

// ── 智能命令建议 ──

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

fn suggest_command(input: &str, mgr: Option<&PluginManager>) {
    let builtins = vec![
        "help", "?", "pwd", "cd", "ls", "dir", "info", "flash", "reset",
        "exit", "quit", "clear", "plugin", "version",
    ];
    let plugin_cmds: Vec<String> = mgr.map(|m| m.all_commands()).unwrap_or_default();
    let all: Vec<&str> = builtins.into_iter()
        .chain(plugin_cmds.iter().map(|s| s.as_str()))
        .collect();

    let mut best: Option<(&str, usize)> = None;
    for cmd in &all {
        let dist = levenshtein(input, cmd);
        if dist <= 3 {
            match best {
                Some((_, d)) if dist < d => best = Some((cmd, dist)),
                None => best = Some((cmd, dist)),
                _ => {}
            }
        }
    }

    println!("{}: {}", t!("未知命令", "Unknown command").red().bold(), input.yellow());
    match best {
        Some((suggestion, _)) => {
            println!("  {} '{}' {}？", t!("你想输入的是", "Did you mean"), suggestion.green().bold(), t!("吗", ""));
        }
        None => {
            println!("  {} '{}' {}。", t!("输入", "Type"), "help".cyan(), t!("查看可用命令", "to see available commands"));
        }
    }
}

// ── cd ──

fn cd(mut parts: std::str::SplitWhitespace, cwd: &PathBuf) -> Option<PathBuf> {
    let target = match parts.next() {
        None => env::var("HOME").ok().map(PathBuf::from)?,
        Some("~") => env::var("HOME").ok().map(PathBuf::from)?,
        Some("-") => env::var("OLDPWD").ok().map(PathBuf::from)?,
        Some(p) if p.starts_with('/') || p.starts_with("./") || p.starts_with("../") || p == ".." => {
            PathBuf::from(p)
        }
        Some(p) => cwd.join(p),
    };
    match target.canonicalize() {
        Ok(p) => Some(p),
        Err(_) => {
            println!("{}", format!("{}: {}", t!("目录不存在", "Directory not found"), target.display()).red());
            None
        }
    }
}

// ── flash / reset ──

fn flash(mut parts: std::str::SplitWhitespace, mgr: Option<&PluginManager>, dl: Option<&str>, cwd: &Path) {
    let file = match parts.next() {
        Some(f) => f,
        None => {
            println!("{}", t!("用法: flash <文件路径>", "Usage: flash <file path>").red());
            println!("  {}: flash firmware.hex", t!("示例", "Example"));
            return;
        }
    };
    let m = match mgr {
        Some(m) => m,
        None => { println!("{}", t!("插件管理器不可用。", "Plugin manager unavailable.").red()); return; }
    };
    let c = dl.and_then(|id| m.find(id)).or_else(|| m.default_downloader());
    match c {
        Some(c) => { run_plugin(c, "flash", &[file.into()], cwd); }
        None => println!("{}", t!("未找到下载器。", "No downloader found.").red()),
    }
}

fn reset(mgr: Option<&PluginManager>, dl: Option<&str>, cwd: &Path) {
    let m = match mgr {
        Some(m) => m,
        None => { println!("{}", t!("插件管理器不可用。", "Plugin manager unavailable.").red()); return; }
    };
    let c = dl.and_then(|id| m.find(id)).or_else(|| m.default_downloader());
    match c {
        Some(c) => { run_plugin(c, "reset", &[], cwd); }
        None => println!("{}", t!("未找到下载器。", "No downloader found.").red()),
    }
}

// ── 插件执行 ──

const FILE_ACTION_NAMES: &[&str] = &["flash", "verify", "compile", "strip"];

fn run_plugin(component: &ComponentInfo, action: &str, args: &[String], cwd: &Path) -> Option<PathBuf> {
    let loader = match find_plugin_loader_tool() {
        Some(p) => p,
        None => { println!("{}", t!("plugin-loader 未找到", "plugin-loader not found").red()); return None; }
    };
    let mut cmd = build_privileged_command(&loader);
    cmd.arg("--manifest")
        .arg(manifest_path().to_string_lossy().as_ref())
        .arg("--component").arg(&component.id)
        .arg("--action").arg(action)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if FILE_ACTION_NAMES.contains(&action) {
        if let Some(f) = args.first() {
            let resolved = if Path::new(f).is_relative() { cwd.join(f) } else { PathBuf::from(f) };
            cmd.arg("--file").arg(resolved.to_string_lossy().as_ref());
            if args.len() > 1 {
                cmd.arg("--");
                for a in &args[1..] { cmd.arg(a); }
            }
        } else {
            println!("{}", tfmt!("{} 需要文件路径", "{} requires a file path", action).red());
            return None;
        }
    } else if !args.is_empty() {
        cmd.arg("--");
        for a in args { cmd.arg(a); }
    }

    println!("{}", tfmt!("执行 {} {}...", "Running {} {}...", component.id, action).cyan());
    match cmd.output() {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() { eprint!("{}", stderr); }
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
                println!("{}", t!("✓ 完成", "✓ Done").green().bold());
            } else {
                println!("{} ({}{})", t!("✗ 失败", "✗ Failed").red().bold(), t!("退出码: ", "exit code: "), output.status.code().unwrap_or(-1));
            }
            cd_target
        }
        Err(e) => {
            println!("{}: {}", t!("错误", "Error").red(), e);
            None
        }
    }
}
