// rabber-stm32-linktool: Rust rewrite for ST-Link V2 helper functionality.
// This binary provides USB device detection, SWD MCU inspection, HEX conversion,
// firmware flashing, and a small interactive shell for user commands.
use clap::Parser;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use which::which;

// USB sysfs root path used to scan connected USB devices.
const SYS_USB_DEVICES: &str = "/sys/bus/usb/devices/";
// ST-Link V2 vendor and product identifiers.
const USB_VENDOR_ID_STLINK_V2: u16 = 0x0483;
const USB_PRODUCT_ID_STLINK_V2: u16 = 0x3748;
const USB_PRODUCT_ID_STLINK_V2_1: u16 = 0x374B;

#[derive(Parser)]
#[command(author, version, about = "Rust 重构的 ST-Link V2 工具", long_about = None)]
struct Args {}

// Cached MCU information obtained from the ST-Link probe.
#[derive(Default)]
struct MCUInfo {
    chip_id: String,
    chip_name: String,
    flash_size: u32,
    core: String,
}

// ST-Link adapter information extracted from st-info CLI.
#[derive(Default)]
struct STLinkInfo {
    version: String,
    serial: String,
    vid: u16,
    pid: u16,
}

// Standardized command execution result used by helper wrappers.
struct CommandResult {
    status: i32,
    stdout: String,
}

// Architecture summary:
// This crate replicates the legacy ST-Link helper in Rust with safe APIs.
// It preserves the original behavior while reducing manual memory handling.
//
// Design principles:
// - Minimize unsafe operations by using Rust standard library abstractions.
// - Keep external tool invocation separated from parsing logic.
// - Provide a simple interactive shell for operators to run commands.
// - Keep dependency surface small and easy to audit.
//
// Core functionality:
// 1) Detect a connected ST-Link V2 device through Linux sysfs.
// 2) Verify the required command-line tools are available.
// 3) Query adapter and MCU metadata using st-info.
// 4) Convert ELF files to Intel HEX when needed.
// 5) Flash the target MCU using st-flash.
// 6) Allow manual reset and information refresh commands.
//
// Comments are focused on critical flows:
// - External tool probing and fallback behavior.
// - File existence checks and cleanup of temporary files.
// - Flash size verification before burning firmware.
//
// Interaction model:
// - The prompt is RabberShell /> and supports history via rustyline.
// - Supported commands include `info`, `flash`, `elf2hex`, `reset`, and `help`.
// - Unknown commands print a clear message and suggest `help`.
//
// Error handling:
// - Any missing binary or failing external invocation is reported clearly.
// - If st-info probe fails, the code attempts secondary methods automatically.
// - Temporary temporary HEX files are cleaned up after use.
//
// Future improvements:
// - Add explicit timeout handling for subprocess execution.
// - Support Windows/macOS with non-sysfs device detection.
// - Add verbose logging levels and structured error output.

fn main() {
    // parse CLI arguments, currently only version/help support via clap
    let _args = Args::parse();

    println!();
    print_banner();

    // Root permission is optional but recommended for USB access and ST-Link operations.
    if !is_root() {
        println!("{}", "[!] 建议以 root 权限运行以获得完整 USB 访问权限".yellow());
    }

    // Verify the required external ST-Link executables are present before doing anything else.
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

    // Detect ST-Link hardware at the USB layer before probing the device.
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

    // Gather ST-Link adapter details and show them to the user.
    let stlink_info = get_stlink_info();
    print_stlink_info(&stlink_info);

    // Attempt a second-level probe through st-info to read MCU details.
    println!("{}", "\n[*] 尝试通过 SWD 读取 MCU 信息...".cyan());
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

// Check whether the current process is running with root privileges.
fn is_root() -> bool {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        if output.status.success() {
            let uid = String::from_utf8_lossy(&output.stdout);
            return uid.trim() == "0";
        }
    }
    false
}

// Run an external program and capture its stdout result.
// This wrapper hides command errors and normalizes exit status values.
// Execute an external command and return its stdout result with status.
// stderr is intentionally ignored in this wrapper because the current
// implementation only needs success/failure and stdout parsing.
fn execute_command(cmd: &str, args: &[&str]) -> CommandResult {
    let output = Command::new(cmd).args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).output();
    match output {
        Ok(output) => CommandResult {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        },
        Err(_) => CommandResult {
            status: -1,
            stdout: String::new(),
        },
    }
}

// Search for a tool in a set of candidate filesystem paths first, then fall back to PATH lookup.
// Locate a helper binary by checking known install paths first,
// then falling back to the shell PATH environment.
fn find_tool(name: &str, possible_paths: &[&str]) -> Option<String> {
    for path in possible_paths {
        if Path::new(path).is_file() && fs::metadata(path).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false) {
            return Some(path.to_string());
        }
    }

    if let Ok(path) = which(name) {
        return Some(path.to_string_lossy().to_string());
    }

    None
}

fn find_stlink_cli_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-info",
        "/usr/local/bin/st-info",
        "/bin/st-info",
        "/usr/bin/stlink-info",
        "/usr/local/bin/stlink-info",
    ];
    find_tool("st-info", &possible_paths)
}

fn find_stlink_programmer_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-flash",
        "/usr/local/bin/st-flash",
        "/bin/st-flash",
        "/usr/bin/stlink-flash",
        "/usr/local/bin/stlink-flash",
    ];
    find_tool("st-flash", &possible_paths)
}

fn check_stlink_tools_installed() -> bool {
    find_stlink_cli_tool().is_some() && find_stlink_programmer_tool().is_some()
}

// Walk the USB sysfs tree and find a device that matches ST-Link V2 identifiers.
// Detect if an ST-Link V2 USB device is currently attached to the host.
fn detect_stlink_by_usb() -> bool {
    let entries = fs::read_dir(SYS_USB_DEVICES);
    if entries.is_err() {
        return false;
    }

    for entry in entries.unwrap().flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }

        let vendor_path = format!("{}/{}/idVendor", SYS_USB_DEVICES.trim_end_matches('/'), name);
        let product_path = format!("{}/{}/idProduct", SYS_USB_DEVICES.trim_end_matches('/'), name);

        if let (Ok(vendor_text), Ok(product_text)) = (fs::read_to_string(&vendor_path), fs::read_to_string(&product_path)) {
            let vendor = vendor_text.trim().trim_start_matches("0x");
            let product = product_text.trim().trim_start_matches("0x");
            if let (Ok(vid), Ok(pid)) = (u16::from_str_radix(vendor, 16), u16::from_str_radix(product, 16)) {
                if vid == USB_VENDOR_ID_STLINK_V2 && (pid == USB_PRODUCT_ID_STLINK_V2 || pid == USB_PRODUCT_ID_STLINK_V2_1) {
                    return true;
                }
            }
        }
    }

    false
}

// Display a fixed ASCII banner on startup.
// Print the startup banner so the user knows the program loaded correctly.
fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           ST-Link V2 MCU 信息读取工具 v1.0           ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
}

// Render ST-Link adapter metadata to the terminal in a boxed layout.
fn print_stlink_info(info: &STLinkInfo) {
    println!("\n{}", "[ ST-Link 信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 型号:      {:<25} │", "ST-Link/V2");
    println!("  │ 版本:      {:<25} │", info.version);
    println!("  │ 序列号:    {:<25} │", if info.serial.is_empty() { "N/A" } else { &info.serial });
    println!("  │ VID/PID:   0x{:04X}/0x{:04X}             │", info.vid, info.pid);
    println!("  └──────────────────────────────────────┘");
}

// Render the detected MCU information in a readable table format.
fn print_mcu_info(info: &MCUInfo) {
    println!("\n{}", "[ 目标MCU信息 ]".magenta());
    println!("  ┌──────────────────────────────────────┐");
    println!("  │ 芯片型号:  {:<25} │", if info.chip_name.is_empty() { "Unknown" } else { &info.chip_name });
    println!("  │ 芯片ID:    0x{:<23} │", if info.chip_id.is_empty() { "N/A" } else { &info.chip_id });
    println!("  │ 内核:      {:<25} │", if info.core.is_empty() { "Unknown" } else { &info.core });
    let flash_kb = info.flash_size / 1024;
    println!("  │ Flash大小: {:<8} KB               │", flash_kb);
    println!("  └──────────────────────────────────────┘");
}

// Start the interactive RabberShell prompt for user commands.
// Launch the interactive command loop, accepting user-entered shell commands.
// The shell retains command history and handles Ctrl+C gracefully.
// Supported commands are intentionally limited to prevent accidental misuse.
// The prompt is RabberShell /> and the UI should remain responsive.
// Command semantics are simple: parse the first token, then execute.
// The interactive mode is designed for embedded workflow operations.
// This shell is not a full scripting language, just a helper interface.
// Extended commands can be added later with minimal parsing changes.
// The shell exits cleanly on EOF or explicit exit/quit commands.

fn interactive_mode() {
    let mut editor = Editor::<(), _>::new().expect("无法初始化交互编辑器");
    loop {
        match editor.readline("RabberShell /> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    // Only store non-empty commands in history.
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

// Parse one user command line and dispatch the correct action.
// Dispatch shell commands after splitting the input string into tokens.
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
                    print_mcu_info(&info);
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

// Query ST-Link adapter metadata using the st-info CLI.
// Collect adapter version and serial details from the st-info utility.
fn get_stlink_info() -> STLinkInfo {
    let mut info = STLinkInfo::default();
    if let Some(cli_path) = find_stlink_cli_tool() {
        let version_output = execute_command(&cli_path, &["--version"]);
        if let Some(pos) = version_output.stdout.find('v') {
            let rest = &version_output.stdout[pos..];
            if let Some(end) = rest.find(|c: char| c.is_whitespace()) {
                info.version = rest[..end].to_string();
            } else {
                info.version = rest.trim().to_string();
            }
        } else {
            info.version = "Unknown".to_string();
        }

        let serial_output = execute_command(&cli_path, &["--serial"]);
        for line in serial_output.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("serial:") || trimmed.starts_with("Serial") {
                if let Some(idx) = trimmed.find(':') {
                    info.serial = trimmed[idx + 1..].trim().to_string();
                    break;
                }
            }
        }
    }
    info.vid = USB_VENDOR_ID_STLINK_V2;
    info.pid = USB_PRODUCT_ID_STLINK_V2;
    info
}

// Convert a flash size string returned by st-info into raw bytes.
// Parse a human-readable flash size string and return a byte count.
fn parse_flash_size(text: &str) -> Option<u32> {
    let text = text.trim();
    let number: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    if number.is_empty() {
        return None;
    }
    let value: u32 = number.parse().ok()?;
    if text.contains("KiB") || text.contains("Ki") {
        Some(value * 1024)
    } else if text.contains("KB") || text.contains("K") {
        Some(value * 1024)
    } else if text.contains("MiB") || text.contains("M") {
        Some(value * 1024 * 1024)
    } else {
        Some(value)
    }
}

// Probe attached MCU details through the ST-Link adapter and st-info.
// Probe the target MCU via SWD using st-info probe commands.
fn get_mcu_info_via_swd() -> MCUInfo {
    let mut info = MCUInfo::default();
    let cli_path = match find_stlink_cli_tool() {
        Some(path) => path,
        None => return info,
    };

    println!();
    print!("{}", "[*] 尝试方法1: st-info 探测...".cyan());
    io::stdout().flush().ok();
    let probe_output = execute_command(&cli_path, &["--probe"]);

    if probe_output.status == 0 && probe_output.stdout.contains("chipid") {
        println!(" {}", "成功".green());
        for line in probe_output.stdout.lines() {
            let line = line.trim();
            if line.starts_with("chipid:") {
                info.chip_id = line[7..].trim().to_string();
            } else if line.starts_with("flash:") {
                if let Some(size) = parse_flash_size(&line[6..]) {
                    info.flash_size = size;
                }
            } else if line.starts_with("dev-type:") {
                info.chip_name = line[9..].trim().to_string();
            }
        }
        info.core = infer_core_from_chip_id(&info.chip_id);
        return info;
    }

    println!(" {}", "失败，尝试方法2...".red());
    print!("{}", "[*] 尝试方法2: 读取 Flash 大小...".cyan());
    io::stdout().flush().ok();
    let flash_output = execute_command(&cli_path, &["--flash"]);
    if flash_output.status == 0 {
        if let Some(size) = parse_flash_size(&flash_output.stdout) {
            info.flash_size = size;
        }
    }

    if info.flash_size > 0 {
        let chipid_output = execute_command(&cli_path, &["--chipid"]);
        if chipid_output.status == 0 {
            info.chip_id = chipid_output.stdout.trim().to_string();
        }
        if info.chip_id.contains("410") || info.chip_id.contains("411") || info.chip_id.contains("412") {
            info.chip_name = "STM32F1 series".to_string();
            info.core = "Cortex-M3".to_string();
        } else if info.chip_id.contains("413") || info.chip_id.contains("423") {
            info.chip_name = "STM32F4 series".to_string();
            info.core = "Cortex-M4".to_string();
        } else if info.chip_id.contains("449") {
            info.chip_name = "STM32F7 series".to_string();
            info.core = "Cortex-M7".to_string();
        } else {
            info.chip_name = "Unknown STM32".to_string();
            info.core = "Cortex-Mx".to_string();
        }
    }

    info
}

// Infer the likely MCU core type based on the observed chip ID prefix.
// Map known chip ID prefixes to expected Cortex-M core variants.
fn infer_core_from_chip_id(chip_id: &str) -> String {
    if chip_id.contains("410") || chip_id.contains("411") || chip_id.contains("412") {
        "Cortex-M3".to_string()
    } else if chip_id.contains("413") || chip_id.contains("423") {
        "Cortex-M4".to_string()
    } else if chip_id.contains("449") {
        "Cortex-M7".to_string()
    } else {
        "Cortex-Mx".to_string()
    }
}

// Convert ELF binary to Intel HEX format using objcopy.
// This is useful when ST-Link flashing requires HEX input.
// Convert an ELF firmware image to Intel HEX format for flashing.
fn elf2hex(elf_file: &str, hex_file: &str) -> Result<(), String> {
    let objcopy = find_tool("arm-none-eabi-objcopy", &["arm-none-eabi-objcopy"]).or_else(|| find_tool("objcopy", &["objcopy"]));
    let objcopy = objcopy.ok_or_else(|| "错误: 找不到 objcopy 工具。请安装 binutils 或 arm-none-eabi-binutils。".to_string())?;

    if !Path::new(elf_file).exists() {
        return Err(format!("错误: 输入文件 '{}' 不存在", elf_file));
    }

    let temp_file = format!("{}.tmp", elf_file);
    let mut source_file = elf_file.to_string();
    let mut cleanup_temp = false;

    // Try stripping debug symbols first to reduce the output size.
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

// Flash a firmware file to the connected MCU using st-flash.
// Flash a file to the target MCU, converting ELF to HEX when required.
fn flash_firmware(file: &str) {
    let flash_path = match find_stlink_programmer_tool() {
        Some(path) => path,
        None => {
            println!("{}", "错误: 找不到 st-flash 工具。".red());
            return;
        }
    };

    // Validate the requested file path before attempting flashing.
    let file_path = Path::new(file);
    if !file_path.exists() {
        println!("{}", format!("错误: 文件 '{}' 不存在。", file).red());
        return;
    }

    let mut actual_file = file.to_string();
    let mut temp_hex: Option<String> = None;
    if file_path.extension().map(|e| e.eq_ignore_ascii_case("elf")).unwrap_or(false) {
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
            if let Some(temp) = temp_hex { let _ = fs::remove_file(temp); }
            return;
        }
    };

    let mcu_info = get_mcu_info_via_swd();
    if mcu_info.flash_size == 0 {
        println!("{}", "错误: 无法获取 MCU Flash 大小。".red());
        if let Some(temp) = temp_hex { let _ = fs::remove_file(temp); }
        return;
    }

    if file_size > mcu_info.flash_size {
        println!("{}", format!("错误: 文件大小 ({}) 已超过 MCU Flash 大小 ({})", file_size, mcu_info.flash_size).red());
        if let Some(temp) = temp_hex { let _ = fs::remove_file(temp); }
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

// Reset the target MCU using st-flash reset command.
// Reset the target MCU using the ST-Link programmer interface.
fn reset_mcu() {
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

// Print user-facing command help in the interactive shell.
// Show the interactive shell help text for available commands.
fn show_help() {
    println!("{}", "可用命令:" .cyan());
    println!("  help          显示此帮助信息");
    println!("  info          查看 MCU 信息");
    println!("  flash <file>  烧录文件到 MCU (支持 ELF 和 HEX)");
    println!("  elf2hex <elf> <hex>  将 ELF 转换为 HEX 格式");
    println!("  reset         复位 MCU");
    println!("  exit/quit     退出交互模式");
}
