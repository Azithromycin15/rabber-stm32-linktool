//! # 输出显示
//!
//! 横幅、设备信息、帮助等格式化输出。

use colored::*;
use crate::plugin::PluginManager;
use crate::stlink::{MCUInfo, STLinkInfo};

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

pub fn print_banner(version: &str) {
    let text = format!("Rabber v{}", version);
    let w = display_width(&text);
    let pad = 15;
    let b = "═".repeat(pad * 2 + w);
    let s = " ".repeat(pad);
    println!("{}", format!("╔{}╗", b).cyan().bold());
    println!("{}", format!("║{}{}{}║", s, text, s).cyan().bold());
    println!("{}", format!("╚{}╝", b).cyan().bold());
}

pub fn print_stlink_info(info: &STLinkInfo) {
    println!("\n{}", "[ ST-Link 信息 ]".magenta().bold());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 型号:      {:<25} │", "ST-Link/V2");
    println!("  │ 版本:      {:<25} │", info.version);
    println!("  │ 序列号:    {:<25} │", if info.serial.is_empty() { "N/A" } else { &info.serial });
    println!("  │ VID/PID:   0x{:04X}/0x{:04X}             │", info.vid, info.pid);
    println!("  └──────────────────────────────────────┘");
}

pub fn print_mcu_info(info: &MCUInfo) {
    let name = if info.chip_name.is_empty() { "Unknown" } else { &info.chip_name };
    let id = if info.chip_id.is_empty() { "N/A" } else { &info.chip_id };
    let core = if info.core.is_empty() { "Unknown" } else { &info.core };
    println!("\n{}", "[ MCU 信息 ]".magenta().bold());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 芯片:      {:<25} │", name);
    println!("  │ ID:        0x{:<23} │", id);
    println!("  │ 内核:      {:<25} │", core);
    println!("  │ Flash:     {:<8} KB               │", info.flash_size / 1024);
    println!("  └──────────────────────────────────────┘");
}

pub fn show_help(mgr: Option<&PluginManager>) {
    println!();
    println!("{}", "┌─ 内置命令 ──────────────────────────────────────────────┐".cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", "│  help, ?        显示帮助 (help <命令> 查看详细用法)       │".cyan());
    println!("{}", "│  pwd            显示当前工作目录                          │".cyan());
    println!("{}", "│  cd  <目录>     切换目录 (支持 ~、-、..、相对/绝对路径)   │".cyan());
    println!("{}", "│  ls, dir  [选项] 列出目录内容 (调用系统 ls)                │".cyan());
    println!("{}", "│  clear          清屏                                      │".cyan());
    println!("{}", "│  info           通过 SWD 查询 MCU 信息                    │".cyan());
    println!("{}", "│  flash <文件>    烧录 ELF/HEX 固件                        │".cyan());
    println!("{}", "│  reset          复位 MCU                                 │".cyan());
    println!("{}", "│  version        显示版本号                                │".cyan());
    println!("{}", "│  exit, quit     退出程序                                  │".cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", "└────────────────────────────────────────────────────────────┘".cyan());

    println!();
    println!("{}", "┌─ 插件管理 ────────────────────────────────────────────────┐".cyan());
    println!("{}", "│  plugin list     列出所有插件                              │".cyan());
    println!("{}", "│  plugin discover 热加载插件清单                            │".cyan());
    println!("{}", "│  plugin refresh  重新探测并刷新插件                        │".cyan());
    println!("{}", "│  plugin help     显示全部插件帮助                          │".cyan());
    println!("{}", "│  <插件ID> help   查看特定插件帮助                          │".cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", "│  用法: <插件ID> <命令> [参数...]                            │".cyan());
    println!("{}", "│  示例: stlink_v2 flash firmware.hex                        │".cyan());
    println!("{}", "│        c_compiler compile main.c --mcu stm32f103c8         │".cyan());
    println!("{}", "└────────────────────────────────────────────────────────────┘".cyan());

    // 列出可用插件
    if let Some(m) = mgr {
        if m.ready() {
            println!();
            println!("{}", "[ 已加载插件 ]".magenta().bold());
            m.list();
        }
    }

    println!();
    println!("{}", "提示: 按 Tab 补全命令和路径 | Ctrl-R 搜索历史 | Ctrl-C 中断".dimmed());
    println!();
}

/// 显示特定命令的详细帮助
pub fn show_command_help(cmd: &str, mgr: Option<&PluginManager>) {
    match cmd {
        "help" | "?" => {
            println!("{}", "help, ?".bold());
            println!("  显示帮助信息。");
            println!();
            println!("  用法:");
            println!("    help              显示所有可用命令");
            println!("    help <命令>       显示特定命令的详细帮助");
            println!("    help plugin       列出所有插件");
            println!();
            println!("  别名: ? 等价于 help");
        }
        "pwd" => {
            println!("{}", "pwd".bold());
            println!("  显示当前工作目录的绝对路径。");
        }
        "cd" => {
            println!("{}", "cd <目录>".bold());
            println!("  切换当前工作目录。");
            println!();
            println!("  用法:");
            println!("    cd           切换到 HOME 目录");
            println!("    cd ~         切换到 HOME 目录");
            println!("    cd -         切换到上一个目录 (OLDPWD)");
            println!("    cd ..        切换到上级目录");
            println!("    cd <路径>    切换到指定路径 (支持相对/绝对路径)");
        }
        "ls" | "dir" => {
            println!("{}", "ls, dir [选项]".bold());
            println!("  列出当前目录内容，直接调用系统 ls 命令。");
            println!();
            println!("  用法:");
            println!("    ls           列出文件");
            println!("    ls -la       详细列表");
            println!("    ls /path     列出指定目录");
            println!();
            println!("  别名: dir 等价于 ls");
        }
        "clear" => {
            println!("{}", "clear".bold());
            println!("  清除终端屏幕。");
            println!();
            println!("  快捷键: 也可以使用 Ctrl-L (终端原生支持)");
        }
        "info" => {
            println!("{}", "info".bold());
            println!("  通过 SWD 接口读取已连接 STM32 MCU 的芯片信息。");
            println!();
            println!("  显示信息包括: 芯片型号、芯片 ID、内核类型、Flash 大小");
            println!();
            println!("  需要: ST-Link 已通过 USB 连接并识别到设备");
        }
        "flash" => {
            println!("{}", "flash <文件>".bold());
            println!("  将固件烧录到已连接的 STM32 MCU。");
            println!();
            println!("  用法:");
            println!("    flash firmware.hex      烧录 HEX 文件");
            println!("    flash firmware.elf      烧录 ELF 文件");
            println!("    flash /path/to/fw.bin   使用绝对路径");
            println!();
            println!("  支持: ELF、HEX、BIN 格式");
            println!("  注意: 烧录完成后自动复位 MCU");
        }
        "reset" => {
            println!("{}", "reset".bold());
            println!("  复位已连接的 STM32 MCU。");
        }
        "exit" | "quit" => {
            println!("{}", "exit, quit".bold());
            println!("  退出 rabber 交互式 Shell。");
            println!();
            println!("  快捷键: Ctrl-D 也可以退出");
        }
        "plugin" => {
            println!("{}", "plugin <子命令>".bold());
            println!("  管理 rabber 插件系统。");
            println!();
            println!("  子命令:");
            println!("    list        列出已加载的插件");
            println!("    discover    从 manifest.yaml 热加载插件列表");
            println!("    refresh     重新扫描 plugins/ 目录并刷新清单");
            println!("    help        显示所有插件的帮助信息");
            println!();
            println!("  快捷别名:");
            println!("    plugin -l   → list");
            println!("    plugin -d   → discover");
            println!("    plugin -r   → refresh");
        }
        "version" => {
            println!("{}", "version".bold());
            println!("  显示 rabber 当前版本号。");
        }
        other => {
            // 检查是否是插件命令
            if let Some(m) = mgr {
                if let Some(c) = m.find_by_command(other) {
                    m.help(&c.id);
                } else {
                    println!("{}", format!("未知命令: {}", other).red());
                    println!("输入 'help' 查看可用命令列表。");
                }
            } else {
                println!("{}", format!("未知命令: {}", other).red());
                println!("输入 'help' 查看可用命令列表。");
            }
        }
    }
}
