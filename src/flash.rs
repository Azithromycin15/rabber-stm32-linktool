//! # 固件烧录模块
//!
//! 这个模块处理固件文件的烧录逻辑，包括 ELF 文件的调试信息剥离、
//! 文件大小验证和通过插件执行实际烧录操作。

use crate::stlink::get_mcu_info_via_swd;
use crate::utils::{execute_command, find_plugin_loader_tool, find_tool};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// 提示用户确认操作
///
/// 显示提示信息并等待用户输入 Y/n。
fn prompt_confirm(prompt: &str) -> bool {
    print!("{} ", prompt.yellow());
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "" | "y" | "yes")
}

/// 剥离 ELF 文件中的调试信息
///
/// 使用 objcopy 工具去除 ELF 文件的调试信息，返回剥离后的文件路径。
fn strip_debug_info(elf_file: &str) -> Result<String, String> {
    let objcopy = find_tool("arm-none-eabi-objcopy", &["arm-none-eabi-objcopy"]).or_else(|| {
        find_tool("objcopy", &["objcopy"])
    });
    let objcopy = objcopy.ok_or_else(|| {
        "错误: 找不到 objcopy 工具。请安装 binutils 或 arm-none-eabi-binutils。".to_string()
    })?;

    if !Path::new(elf_file).exists() {
        return Err(format!("错误: 输入文件 '{}' 不存在", elf_file));
    }

    let stripped_file = format!("{}.stripped.elf", elf_file);
    println!("{}", "检测到 ELF 文件，正在去除调试信息...".cyan());
    let strip_result = execute_command(&objcopy, &["--strip-debug", elf_file, &stripped_file]);
    if strip_result.status != 0 {
        let _ = fs::remove_file(&stripped_file);
        return Err(format!(
            "错误: 无法去除 ELF 文件调试信息 (exit {})",
            strip_result.status
        ));
    }

    if !Path::new(&stripped_file).exists() {
        return Err(format!("错误: 输出文件 '{}' 未生成", stripped_file));
    }

    Ok(stripped_file)
}

/// 烧录固件到 MCU
///
/// 处理 ELF 文件的预处理（剥离调试信息）、文件验证、
/// 用户确认，并通过插件执行实际烧录操作。
pub fn flash_firmware(file: &str) -> bool {
    let file_path = Path::new(file);
    if !file_path.exists() {
        println!("{}", format!("错误: 文件 '{}' 不存在。", file).red());
        return false;
    }

    let mut actual_file = file.to_string();
    let mut cleanup_temp: Option<String> = None;
    if file_path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("elf"))
        .unwrap_or(false)
    {
        match strip_debug_info(file) {
            Ok(stripped) => {
                actual_file = stripped.clone();
                cleanup_temp = Some(stripped);
            }
            Err(err) => {
                println!("{}", err.red());
                return false;
            }
        }
    }

    let mcu_info = get_mcu_info_via_swd();
    if mcu_info.flash_size == 0 {
        println!("{}", "错误: 无法获取 MCU Flash 大小。".red());
        if let Some(temp) = cleanup_temp {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    let file_size = match fs::metadata(&actual_file) {
        Ok(meta) => meta.len() as u32,
        Err(_) => {
            println!("{}", "错误: 无法获取文件大小。".red());
            if let Some(temp) = cleanup_temp {
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
        if let Some(temp) = cleanup_temp {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    println!("{}", format!("目标设备 Flash 大小: {} KB", mcu_info.flash_size / 1024).cyan());
    println!("{}", format!("待烧录文件: {} ({} 字节)", actual_file, file_size).cyan());

    if !prompt_confirm("是否继续烧录并复位 MCU？[Y/n]") {
        println!("{}", "已取消烧录。".yellow());
        if let Some(temp) = cleanup_temp {
            let _ = fs::remove_file(temp);
        }
        return false;
    }

    let plugin_loader = match find_plugin_loader_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 plugin-loader 二进制。请先构建 plugin-loader。".red());
            if let Some(temp) = cleanup_temp {
                let _ = fs::remove_file(temp);
            }
            return false;
        }
    };

    println!("{}", format!("正在使用插件烧录 '{}'...", actual_file).cyan());
    let status = Command::new(&plugin_loader)
        .arg("--manifest")
        .arg("plugins/manifest.yaml")
        .arg("--component")
        .arg("stlink_v2")
        .arg("--action")
        .arg("flash")
        .arg("--file")
        .arg(&actual_file)
        .status();

    let success = match status {
        Ok(status) if status.success() => true,
        Ok(_) => {
            println!("{}", "错误: 插件烧录失败。".red());
            false
        }
        Err(err) => {
            println!("{}", format!("错误: 无法执行 plugin-loader: {}", err).red());
            false
        }
    };

    if let Some(temp) = cleanup_temp {
        let _ = fs::remove_file(temp);
    }

    success
}

/// 复位 MCU
///
/// 通过插件执行 MCU 复位操作。
pub fn reset_mcu() {
    println!("{}", "正在通过插件复位 MCU...".cyan());
    let plugin_loader = match find_plugin_loader_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 plugin-loader 二进制。请先构建 plugin-loader。".red());
            return;
        }
    };

    let status = Command::new(&plugin_loader)
        .arg("--manifest")
        .arg("plugins/manifest.yaml")
        .arg("--component")
        .arg("stlink_v2")
        .arg("--action")
        .arg("reset")
        .status();

    match status {
        Ok(status) if status.success() => println!("{}", "复位成功。".green()),
        Ok(_) => println!("{}", "复位失败。".red()),
        Err(err) => println!("{}", format!("复位失败: {}", err).red()),
    }
}
