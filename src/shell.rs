//! # 交互式 Shell 模块
//!
//! 这个模块实现了一个交互式的命令行界面，支持内置命令和插件命令的执行。

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::output::show_help;
use crate::plugin::PluginManager;
use crate::stlink::get_mcu_info_via_swd;
use crate::utils::find_plugin_loader_tool;

/// 启动交互模式
///
/// 初始化 rustyline 编辑器并进入命令循环，处理用户输入的命令。
pub fn interactive_mode(plugin_manager: Option<PluginManager>, default_downloader: Option<String>) {
    let mut editor = Editor::<(), _>::new().expect("无法初始化交互编辑器");
    loop {
        match editor.readline("RabberShell /> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    editor.add_history_entry(trimmed).ok();
                    handle_command(trimmed, plugin_manager.as_ref(), default_downloader.as_deref());
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("读取输入失败: {}", err);
                break;
            }
        }
    }
}

/// 处理命令
///
/// 解析并执行用户输入的命令，支持内置命令和插件命令。
fn handle_command(line: &str, plugin_manager: Option<&PluginManager>, default_downloader: Option<&str>) {
    let mut parts = line.split_whitespace();
    if let Some(command) = parts.next() {
        match command {
            "exit" | "quit" => {
                println!("退出交互模式。");
                std::process::exit(0);
            }
            "help" => {
                if let Some(plugin_id) = parts.next() {
                    if let Some(manager) = plugin_manager {
                        manager.print_component_help(plugin_id);
                    } else {
                        println!("{}", "未加载插件清单，无法显示插件命令。".yellow());
                    }
                } else {
                    show_help();
                }
            }
            "info" => {
                let info = get_mcu_info_via_swd();
                if !info.chip_id.is_empty() {
                    crate::output::print_mcu_info(&info);
                } else {
                    println!("{}", "无法获取 MCU 信息。".red());
                }
            }
            "flash" => {
                if let Some(file) = parts.next() {
                    if let Some(manager) = plugin_manager {
                        let component = default_downloader
                            .and_then(|id| manager.find_component(id))
                            .or_else(|| manager.default_downloader_component());
                        if let Some(component) = component {
                            let args = vec![file.to_string()];
                            execute_plugin_command(component, "flash", &args);
                        } else {
                            println!("{}", "未找到默认下载器组件。".red());
                        }
                    } else {
                        println!("{}", "插件管理器未初始化。".red());
                    }
                } else {
                    println!("{}", "错误: flash 命令需要指定 ELF 或 HEX 文件路径。".red());
                    println!("用法: flash <file>");
                }
            }
            "reset" => {
                if let Some(manager) = plugin_manager {
                    let component = default_downloader
                        .and_then(|id| manager.find_component(id))
                        .or_else(|| manager.default_downloader_component());
                    if let Some(component) = component {
                        execute_plugin_command(component, "reset", &[]);
                    } else {
                        println!("{}", "未找到默认下载器组件。".red());
                    }
                } else {
                    println!("{}", "插件管理器未初始化。".red());
                }
            }
            plugin_id => {
                if let Some(manager) = plugin_manager {
                    if let Some(component) = manager.find_component(plugin_id) {
                        if let Some(action) = parts.next() {
                            if action == "help" {
                                manager.print_component_help(plugin_id);
                                return;
                            }
                            if !manager.has_action(plugin_id, action) {
                                println!("{}", format!("插件 '{}' 不支持命令 '{}'。", plugin_id, action).red());
                                manager.print_component_help(plugin_id);
                                return;
                            }
                            let args: Vec<String> = parts.map(|s| s.to_string()).collect();
                            execute_plugin_command(component, action, &args);
                        } else {
                            println!("{}", "请输入插件命令。例如: <plugin_id> <command> [options]".yellow());
                            manager.print_component_help(plugin_id);
                        }
                    } else {
                        println!("{}: {}", "未知命令".red(), command);
                        println!("输入 'help' 查看可用命令。");
                    }
                } else {
                    println!("{}: {}", "未知命令".red(), command);
                    println!("输入 'help' 查看可用命令。" );
                }
            }
        }
    }
}

/// 执行插件命令
///
/// 通过 plugin-loader 调用指定的插件组件和动作。
fn execute_plugin_command(component: &crate::plugin::ComponentInfo, action: &str, args: &[String]) {
    use std::process::Command;

    let loader_path = match find_plugin_loader_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 无法找到 plugin-loader 二进制。请先构建 plugin-loader。".red());
            return;
        }
    };

    let mut command = Command::new(loader_path);
    command.arg("--manifest").arg("plugins/manifest.yaml");
    command.arg("--component").arg(&component.id);
    command.arg("--action").arg(action);

    if action == "flash" {
        if let Some(file) = args.first() {
            command.arg("--file").arg(file);
        } else {
            println!("{}", "错误: flash 命令需要指定文件路径。".red());
            return;
        }
    }

    println!("{}", format!("调用插件 {} 的命令 '{}'...", component.id, action).cyan());
    match command.status() {
        Ok(status) if status.success() => println!("{}", "插件命令执行成功。".green()),
        Ok(_) => println!("{}", "插件命令执行失败。".red()),
        Err(err) => println!("{}", format!("无法执行 plugin-loader: {}", err).red()),
    }
}
