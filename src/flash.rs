use crate::stlink::get_mcu_info_via_swd;
use crate::utils::{execute_command, find_tool, find_stlink_programmer_tool};
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;

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

pub fn flash_firmware(file: &str) {
    let flash_path = match find_stlink_programmer_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 st-flash 工具。".red());
            return;
        }
    };

    let file_path = Path::new(file);
    if !file_path.exists() {
        println!("{}", format!("错误: 文件 '{}' 不存在。", file).red());
        return;
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
            return;
        }
        actual_file = hex_file.clone();
        temp_hex = Some(hex_file);
    }

    let file_size = match fs::metadata(&actual_file) {
        Ok(meta) => meta.len() as u32,
        Err(_) => {
            println!("{}", "错误: 无法获取文件大小。".red());
            if let Some(temp) = temp_hex {
                let _ = fs::remove_file(temp);
            }
            return;
        }
    };

    let mcu_info = get_mcu_info_via_swd();
    if mcu_info.flash_size == 0 {
        println!("{}", "错误: 无法获取 MCU Flash 大小。".red());
        if let Some(temp) = temp_hex {
            let _ = fs::remove_file(temp);
        }
        return;
    }

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
        return;
    }

    println!("{}", format!("正在烧录 '{}' (大小: {} 字节)...", file, file_size).cyan());
    let status = Command::new(&flash_path)
        .arg("write")
        .arg(&actual_file)
        .arg("0x8000000")
        .status();

    match status {
        Ok(status) if status.success() => println!("{}", "烧录成功。".green()),
        Ok(_) => println!("{}", "烧录失败。".red()),
        Err(err) => println!("{}", format!("烧录失败: {}", err).red()),
    }

    if let Some(temp) = temp_hex {
        let _ = fs::remove_file(temp);
    }
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
