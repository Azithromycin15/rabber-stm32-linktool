mod cli;
mod flash;
mod output;
mod shell;
mod stlink;
mod utils;

use clap::Parser;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

use cli::Args;
use output::{print_banner, print_mcu_info, print_stlink_info};
use shell::interactive_mode;
use stlink::{detect_stlink_by_usb, get_mcu_info_via_swd, get_stlink_info};
use utils::{check_stlink_tools_installed, is_root};

fn main() {
    let _args = Args::parse();

    println!();
    print_banner();

    if !is_root() {
        println!("{}", "[!] 建议以 root 权限运行以获得完整 USB 访问权限".yellow());
    }

    print!("{}", "[*] 检查 ST-Link 工具链...".cyan());
    io::stdout().flush().ok();
    if !check_stlink_tools_installed() {
        println!(" {}", "未找到".red());
        println!("{}", "[!] 请先安装 stlink-tools:".yellow());
        println!("    Ubuntu/Debian: sudo apt-get install stlink-tools");
        println!("    Arch: sudo pacman -S stlink");
        println!("    或从源码编译: https://github.com/stlink-org/stlink");
        return;
    }
    println!(" {}", "已安装".green());

    print!("{}", "[*] 扫描 USB 设备...".cyan());
    io::stdout().flush().ok();
    if !detect_stlink_by_usb() {
        println!(" {}", "未检测到 ST-Link 设备".red());
        println!("{}", "[!] 尝试列出所有 USB 设备...".yellow());
        let _ = Command::new("sh")
            .arg("-c")
            .arg("lsusb | grep -i stm32 || lsusb | grep -i st-link")
            .status();
        return;
    }
    println!(" {}", "检测到 ST-Link 设备".green());

    let stlink_info = get_stlink_info();
    print_stlink_info(&stlink_info);

    println!("\n{}", "[*] 尝试通过 SWD 读取 MCU 信息...".cyan());
    let mcu_info = get_mcu_info_via_swd();
    if !mcu_info.chip_id.is_empty() {
        println!(" {}", "成功".green());
        print_mcu_info(&mcu_info);
    } else {
        println!(" {}", "失败".red());
        println!("{}", "[!] 可能的原因:" .yellow());
        println!("    1. 目标板未正确连接或未供电");
        println!("    2. SWD 接口连接错误");
        println!("    3. 目标 MCU 处于休眠/复位状态");
        println!("    4. SWD 接口被禁用（尝试先擦除 Flash）");
    }

    println!("\n{}", "[✓] 检测完成，进入交互模式".green());
    println!("{}", "[!] 提示: 可以使用 'help' 命令获取支持的指令信息".yellow());

    interactive_mode();
}
