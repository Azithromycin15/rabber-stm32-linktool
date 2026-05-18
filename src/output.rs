//! # 输出显示
//!
//! 横幅、设备信息、帮助等格式化输出。

use colored::*;
use crate::plugin::PluginManager;
use crate::stlink::{MCUInfo, STLinkInfo};
use crate::t;

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
    println!("\n{}", format!("[ {} ]", t!("ST-Link 信息", "ST-Link Info")).magenta().bold());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ {}:     {:<25} │", t!("型号", "Model"), "ST-Link/V2");
    println!("  │ {}:     {:<25} │", t!("版本", "Version"), info.version);
    println!("  │ {}:   {:<25} │", t!("序列号", "Serial"), if info.serial.is_empty() { "N/A" } else { &info.serial });
    println!("  │ VID/PID:   0x{:04X}/0x{:04X}             │", info.vid, info.pid);
    println!("  └──────────────────────────────────────┘");
}

pub fn print_mcu_info(info: &MCUInfo) {
    let name = if info.chip_name.is_empty() { "Unknown" } else { &info.chip_name };
    let id = if info.chip_id.is_empty() { "N/A" } else { &info.chip_id };
    let core = if info.core.is_empty() { "Unknown" } else { &info.core };
    println!("\n{}", format!("[ {} ]", t!("MCU 信息", "MCU Info")).magenta().bold());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ {}:      {:<25} │", t!("芯片", "Chip"), name);
    println!("  │ ID:        0x{:<23} │", id);
    println!("  │ {}:      {:<25} │", t!("内核", "Core"), core);
    println!("  │ Flash:     {:<8} KB               │", info.flash_size / 1024);
    println!("  └──────────────────────────────────────┘");
}

pub fn show_help(mgr: Option<&PluginManager>) {
    println!();
    println!("{}", format!("┌─ {} ──────────────────────────────────────────────┐", t!("内置命令", "Built-in Commands")).cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", format!("│  help, ?        {}       │", t!("显示帮助 (help <命令> 查看详细用法)", "Show help (help <cmd> for details)")).cyan());
    println!("{}", format!("│  pwd            {}                          │", t!("显示当前工作目录", "Print working directory")).cyan());
    println!("{}", format!("│  cd  <{}>     {}   │", t!("目录", "dir"), t!("切换目录 (支持 ~、-、..、相对/绝对路径)", "Change directory (supports ~, -, ..)")).cyan());
    println!("{}", format!("│  ls, dir  [{}] {}                │", t!("选项", "opts"), t!("列出目录内容 (调用系统 ls)", "List directory (calls system ls)")).cyan());
    println!("{}", format!("│  clear          {}                                      │", t!("清屏", "Clear screen")).cyan());
    println!("{}", format!("│  info           {}                    │", t!("通过 SWD 查询 MCU 信息", "Query MCU info via SWD")).cyan());
    println!("{}", format!("│  flash <{}>    {}                        │", t!("文件", "file"), t!("烧录 ELF/HEX 固件", "Flash ELF/HEX firmware")).cyan());
    println!("{}", format!("│  reset          {}                                 │", t!("复位 MCU", "Reset MCU")).cyan());
    println!("{}", format!("│  version        {}                                │", t!("显示版本号", "Show version")).cyan());
    println!("{}", format!("│  exit, quit     {}                                  │", t!("退出程序", "Exit program")).cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", format!("└────────────────────────────────────────────────────────────┘").cyan());

    println!();
    println!("{}", format!("┌─ {} ────────────────────────────────────────────────┐", t!("插件管理", "Plugin Management")).cyan());
    println!("{}", format!("│  plugin list     {}                              │", t!("列出所有插件", "List all plugins")).cyan());
    println!("{}", format!("│  plugin discover {}                            │", t!("热加载插件清单", "Hot-reload manifest")).cyan());
    println!("{}", format!("│  plugin refresh  {}                        │", t!("重新探测并刷新插件", "Re-probe and refresh plugins")).cyan());
    println!("{}", format!("│  plugin help     {}                          │", t!("显示全部插件帮助", "Show all plugin help")).cyan());
    println!("{}", format!("│  <{}> help   {}                          │", t!("插件ID", "pluginID"), t!("查看特定插件帮助", "View specific plugin help")).cyan());
    println!("{}", "│                                                            │".cyan());
    println!("{}", format!("│  {}: <{}> <{}> [{}...]                            │", t!("用法", "Usage"), t!("插件ID", "pluginID"), t!("命令", "command"), t!("参数", "args")).cyan());
    println!("{}", format!("│  {}: stlink_v2 flash firmware.hex                        │", t!("示例", "Example")).cyan());
    println!("{}", format!("│        c_compiler compile main.c --mcu stm32f103c8         │").cyan());
    println!("{}", format!("└────────────────────────────────────────────────────────────┘").cyan());

    if let Some(m) = mgr {
        if m.ready() {
            println!();
            println!("{}", format!("[ {} ]", t!("已加载插件", "Loaded Plugins")).magenta().bold());
            m.list();
        }
    }

    println!();
    println!("{}", t!("提示: 按 Tab 补全命令和路径 | Ctrl-R 搜索历史 | Ctrl-C 中断",
        "Tip: Tab to complete | Ctrl-R search history | Ctrl-C to interrupt").dimmed());
    println!();
}

/// 显示特定命令的详细帮助
pub fn show_command_help(cmd: &str, mgr: Option<&PluginManager>) {
    match cmd {
        "help" | "?" => {
            println!("{}", "help, ?".bold());
            println!("  {}", t!("显示帮助信息。", "Show help information."));
            println!();
            println!("  {}:", t!("用法", "Usage"));
            println!("    help              {}", t!("显示所有可用命令", "Show all available commands"));
            println!("    help <{}>       {}", t!("命令", "command"), t!("显示特定命令的详细帮助", "Show detailed help for a command"));
            println!("    help plugin       {}", t!("列出所有插件", "List all plugins"));
            println!();
            println!("  {}: ? {} help", t!("别名", "Alias"), t!("等价于", "equivalent to"));
        }
        "pwd" => {
            println!("{}", "pwd".bold());
            println!("  {}", t!("显示当前工作目录的绝对路径。", "Print the absolute path of the current working directory."));
        }
        "cd" => {
            println!("{}", format!("cd <{}>", t!("目录", "directory")).bold());
            println!("  {}", t!("切换当前工作目录。", "Change the current working directory."));
            println!();
            println!("  {}:", t!("用法", "Usage"));
            println!("    cd           {}", t!("切换到 HOME 目录", "Change to HOME directory"));
            println!("    cd ~         {}", t!("切换到 HOME 目录", "Change to HOME directory"));
            println!("    cd -         {}", t!("切换到上一个目录 (OLDPWD)", "Change to previous directory (OLDPWD)"));
            println!("    cd ..        {}", t!("切换到上级目录", "Change to parent directory"));
            println!("    cd <{}>    {}", t!("路径", "path"), t!("切换到指定路径 (支持相对/绝对路径)", "Change to specified path (relative/absolute)"));
        }
        "ls" | "dir" => {
            println!("{}", format!("ls, dir [{}]", t!("选项", "options")).bold());
            println!("  {}", t!("列出当前目录内容，直接调用系统 ls 命令。", "List directory contents, calls system ls command."));
            println!();
            println!("  {}:", t!("用法", "Usage"));
            println!("    ls           {}", t!("列出文件", "List files"));
            println!("    ls -la       {}", t!("详细列表", "Detailed list"));
            println!("    ls /path     {}", t!("列出指定目录", "List specified directory"));
            println!();
            println!("  {}: dir {} ls", t!("别名", "Alias"), t!("等价于", "equivalent to"));
        }
        "clear" => {
            println!("{}", "clear".bold());
            println!("  {}", t!("清除终端屏幕。", "Clear the terminal screen."));
            println!();
            println!("  {}: {} Ctrl-L ({}原生支持)", t!("快捷键", "Shortcut"), t!("也可以使用", "also available via"), t!("终端", "terminal"));
        }
        "info" => {
            println!("{}", "info".bold());
            println!("  {}", t!("通过 SWD 接口读取已连接 STM32 MCU 的芯片信息。", "Read chip information of connected STM32 MCU via SWD."));
            println!();
            println!("  {}: {}, {}, {}, Flash {}", t!("显示信息包括", "Displays"), t!("芯片型号", "chip model"), t!("芯片 ID", "chip ID"), t!("内核类型", "core type"), t!("大小", "size"));
            println!();
            println!("  {}: ST-Link {} USB {}", t!("需要", "Requires"), t!("已通过", "connected via"), t!("连接并识别到设备", "and device detected"));
        }
        "flash" => {
            println!("{}", format!("flash <{}>", t!("文件", "file")).bold());
            println!("  {}", t!("将固件烧录到已连接的 STM32 MCU。", "Flash firmware to the connected STM32 MCU."));
            println!();
            println!("  {}:", t!("用法", "Usage"));
            println!("    flash firmware.hex      {}", t!("烧录 HEX 文件", "Flash HEX file"));
            println!("    flash firmware.elf      {}", t!("烧录 ELF 文件", "Flash ELF file"));
            println!("    flash /path/to/fw.bin   {}", t!("使用绝对路径", "Use absolute path"));
            println!();
            println!("  {}: ELF、HEX、BIN {}", t!("支持", "Supports"), t!("格式", "format"));
            println!("  {}: {}", t!("注意", "Note"), t!("烧录完成后自动复位 MCU", "MCU will be reset after flashing"));
        }
        "reset" => {
            println!("{}", "reset".bold());
            println!("  {}", t!("复位已连接的 STM32 MCU。", "Reset the connected STM32 MCU."));
        }
        "exit" | "quit" => {
            println!("{}", "exit, quit".bold());
            println!("  {} rabber {} Shell。", t!("退出", "Exit"), t!("交互式", "interactive"));
            println!();
            println!("  {}: Ctrl-D {}", t!("快捷键", "Shortcut"), t!("也可以退出", "also exits"));
        }
        "plugin" => {
            println!("{}", format!("plugin <{}>", t!("子命令", "subcommand")).bold());
            println!("  {} rabber {}。", t!("管理", "Manage"), t!("插件系统", "plugin system"));
            println!();
            println!("  {}:", t!("子命令", "Subcommands"));
            println!("    list        {}", t!("列出已加载的插件", "List loaded plugins"));
            println!("    discover    {} manifest.yaml {}", t!("从", "from"), t!("热加载插件列表", "hot-reload plugin list"));
            println!("    refresh     {} plugins/ {}", t!("重新扫描", "Rescan"), t!("目录并刷新清单", "dir and refresh manifest"));
            println!("    help        {}", t!("显示所有插件的帮助信息", "Show help for all plugins"));
            println!();
            println!("  {}:", t!("快捷别名", "Short aliases"));
            println!("    plugin -l   → list");
            println!("    plugin -d   → discover");
            println!("    plugin -r   → refresh");
        }
        "version" => {
            println!("{}", "version".bold());
            println!("  {} rabber {}。", t!("显示", "Show"), t!("当前版本号", "current version number"));
        }
        other => {
            if let Some(m) = mgr {
                if let Some(c) = m.find_by_command(other) {
                    m.help(&c.id);
                } else {
                    println!("{}", format!("{}: {}", t!("未知命令", "Unknown command").red(), other));
                    println!("{} 'help' {}。", t!("输入", "Type"), t!("查看可用命令列表", "to see available commands"));
                }
            } else {
                println!("{}", format!("{}: {}", t!("未知命令", "Unknown command").red(), other));
                println!("{} 'help' {}。", t!("输入", "Type"), t!("查看可用命令列表", "to see available commands"));
            }
        }
    }
}
