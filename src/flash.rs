use crate::output::print_mcu_info;
use crate::stlink::get_mcu_info_via_swd;
use crate::utils::{execute_command, find_tool, find_stlink_programmer_tool};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const FLASH_START_ADDRESS: &str = "0x08000000";

fn prompt_confirm(prompt: &str) -> bool {
    print!("{} ", prompt.yellow());
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "" | "y" | "yes")
}

pub fn elf2hex(elf_file: &str, hex_file: &str) -> Result<(), String> {
    let objcopy = find_tool("arm-none-eabi-objcopy", &["arm-none-eabi-objcopy"]).or_else(|| {
        find_tool("objcopy", &["objcopy"])
    });
    let objcopy = objcopy.ok_or_else(|| {
        "错误: 找不到 objcopy 工具。请安装 binutils 或 arm-none-eabi-binutils。".to_string()
    })?;

    if !Path::new(elf_file).exists() {
        return Err(format!("错误: 输入文件 '{}' 不存在", elf_file));
    }

    let temp_file = format!("{}.tmp", elf_file);
    let mut source_file = elf_file.to_string();
    let mut cleanup_temp = false;

    println!("{}", "正在去除调试信息...".cyan());
    let strip_result = execute_command(&objcopy, &["--strip-debug", elf_file, &temp_file]);
    if strip_result.status == 0 {
        source_file = temp_file.clone();
        cleanup_temp = true;
    } else {
        println!("{}", "去除调试信息失败，尝试直接转换原文件...".yellow());
    }

    println!("{}", "正在转换为 HEX 格式...".cyan());
    let convert_result = execute_command(&objcopy, &["-O", "ihex", &source_file, hex_file]);
    if convert_result.status != 0 {
        if cleanup_temp {
            let _ = fs::remove_file(&temp_file);
        }
        return Err(format!("错误: 转换为 HEX 格式失败 (exit {})", convert_result.status));
    }

    if cleanup_temp {
        let _ = fs::remove_file(&temp_file);
    }

    if !Path::new(hex_file).exists() {
        return Err(format!("错误: 输出文件 '{}' 未生成", hex_file));
    }

    println!("{} -> {}", elf_file, hex_file);
    Ok(())
}

pub fn flash_firmware(file: &str) -> bool {
    let flash_path = match find_stlink_programmer_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 st-flash 工具。".red());
            return false;
        }
    };

    let file_path = Path::new(file);
    if !file_path.exists() {
        println!("{}", format!("错误: 文件 '{}' 不存在。", file).red());
        return false;
    }

    let mut actual_file = file.to_string();
    let mut temp_hex: Option<String> = None;
    if file_path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("elf"))
        .unwrap_or(false)
    {
        let hex_file = format!("{}.hex", file);
        println!("{}", "检测到 ELF 文件，正在转换为 HEX...".cyan());
        if let Err(err) = elf2hex(file, &hex_file) {
            println!("{}", err.red());
            return false;
        }
        actual_file = hex_file.clone();
        temp_hex = Some(hex_file);
    }

    let mcu_info = get_mcu_info_via_swd();
    if mcu_info.flash_size == 0 {
        println!("{}", "错误: 无法获取 MCU Flash 大小。".red());
        if let Some(temp) = temp_hex {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    let file_size = match fs::metadata(&actual_file) {
        Ok(meta) => meta.len() as u32,
        Err(_) => {
            println!("{}", "错误: 无法获取文件大小。".red());
            if let Some(temp) = temp_hex {
                let _ = fs::remove_file(temp);
            }
            return false;
        }
    };

    if file_size > mcu_info.flash_size {
        println!(
            "{}",
            format!(
                "错误: 文件大小 ({}) 已超过 MCU Flash 大小 ({})",
                file_size, mcu_info.flash_size
            )
            .red()
        );
        if let Some(temp) = temp_hex {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    println!("{}", format!("目标设备 Flash 大小: {} KB", mcu_info.flash_size / 1024).cyan());
    println!("{}", format!("待烧录文件: {} ({} 字节)", actual_file, file_size).cyan());

    if !prompt_confirm("是否继续烧录并复位 MCU？[Y/n]") {
        println!("{}", "已取消烧录。".yellow());
        if let Some(temp) = temp_hex {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    println!("{}", format!("正在烧录 '{}'...", actual_file).cyan());
    let status = Command::new(&flash_path)
        .arg("write")
        .arg(&actual_file)
        .arg(FLASH_START_ADDRESS)
        .status();

    let success = match status {
        Ok(status) if status.success() => {
            println!("{}", "烧录成功。".green());
            true
        }
        Ok(_) => {
            println!("{}", "烧录失败。".red());
            false
        }
        Err(err) => {
            println!("{}", format!("烧录失败: {}", err).red());
            false
        }
    };

    if success {
        println!("{}", "正在复位 MCU 以启动程序...".cyan());
        reset_mcu();
        println!("{}", "正在验证 MCU 启动状态...".cyan());
        let verify_info = get_mcu_info_via_swd();
        if !verify_info.chip_id.is_empty() {
            println!("{}", "烧录并复位完成，MCU 已成功响应 SWD。".green());
            print_mcu_info(&verify_info);
        } else {
            println!("{}", "警告: MCU 复位后未能通过 SWD 验证。请检查程序是否已正确启动、目标板电源和 SWD 连接。".yellow());
        }
    }

    if let Some(temp) = temp_hex {
        let _ = fs::remove_file(temp);
    }

    success
}

pub fn reset_mcu() {
    let flash_path = match find_stlink_programmer_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 st-flash 工具。".red());
            return;
        }
    };

    println!("{}", "正在复位 MCU...".cyan());
    let status = Command::new(&flash_path).arg("reset").status();
    match status {
        Ok(status) if status.success() => println!("{}", "复位成功。".green()),
        Ok(_) => println!("{}", "复位失败。".red()),
        Err(err) => println!("{}", format!("复位失败: {}", err).red()),
    }
}
