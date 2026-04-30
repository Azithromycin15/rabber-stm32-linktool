use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::flash::{elf2hex, flash_firmware, reset_mcu};
use crate::output::show_help;
use crate::stlink::get_mcu_info_via_swd;

pub fn interactive_mode() {
    let mut editor = Editor::<(), _>::new().expect("无法初始化交互编辑器");
    loop {
        match editor.readline("RabberShell /> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    editor.add_history_entry(trimmed).ok();
                    handle_command(trimmed);
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

fn handle_command(line: &str) {
    let mut parts = line.split_whitespace();
    if let Some(command) = parts.next() {
        match command {
            "exit" | "quit" => {
                println!("退出交互模式。");
                std::process::exit(0);
            }
            "help" => show_help(),
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
                    flash_firmware(file);
                } else {
                    println!("{}", "错误: flash 命令需要指定 ELF 或 HEX 文件路径。".red());
                    println!("用法: flash <file>");
                }
            }
            "elf2hex" => {
                let elf_file = parts.next();
                let hex_file = parts.next();
                if let (Some(elf), Some(hex)) = (elf_file, hex_file) {
                    if elf2hex(elf, hex).is_ok() {
                        println!("{}", "转换完成。".green());
                    }
                } else {
                    println!("{}", "错误: elf2hex 命令需要输入 ELF 文件和输出 HEX 文件路径。".red());
                    println!("用法: elf2hex <elf_file> <hex_file>");
                }
            }
            "reset" => reset_mcu(),
            _ => {
                println!("{}: {}", "未知命令".red(), command);
                println!("输入 'help' 查看可用命令。");
            }
        }
    }
}
