use colored::Colorize;
use crate::plugin::PluginManager;
use crate::utils::{execute_command, find_stlink_cli_tool};
use std::fs;
use std::io::{self, Write};

pub const SYS_USB_DEVICES: &str = "/sys/bus/usb/devices/";

#[derive(Default)]
pub struct MCUInfo {
    pub chip_id: String,
    pub chip_name: String,
    pub flash_size: u32,
    pub core: String,
}

#[derive(Default)]
pub struct STLinkInfo {
    pub version: String,
    pub serial: String,
    pub vid: u16,
    pub pid: u16,
}

fn parse_hex_id(id: &str) -> Option<u16> {
    let normalized = id.trim().trim_start_matches("0x");
    u16::from_str_radix(normalized, 16).ok()
}

fn default_stlink_metadata() -> Option<(u16, Vec<u16>)> {
    if let Some(manager) = PluginManager::load_from("plugins/manifest.yaml") {
        if let Some(component) = manager.default_stlink_component() {
            let vendor_id = parse_hex_id(&component.metadata.vendor_id)?;
            let product_ids: Vec<u16> = component
                .metadata
                .product_ids
                .iter()
                .filter_map(|p| parse_hex_id(p))
                .collect();
            return Some((vendor_id, product_ids));
        }
    }
    None
}

pub fn detect_stlink_by_usb() -> bool {
    let (vendor_id, product_ids) = match default_stlink_metadata() {
        Some((vendor_id, product_ids)) => (vendor_id, product_ids),
        None => (0x0483, vec![0x3748, 0x374B]),
    };

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
                if vid == vendor_id && product_ids.contains(&pid) {
                    return true;
                }
            }
        }
    }

    false
}

pub fn get_stlink_info() -> STLinkInfo {
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
    if let Some((vendor_id, product_ids)) = default_stlink_metadata() {
        info.vid = vendor_id;
        info.pid = *product_ids.get(0).unwrap_or(&0);
    } else {
        info.vid = 0x0483;
        info.pid = 0x3748;
    }
    info
}

pub fn parse_flash_size(text: &str) -> Option<u32> {
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

pub fn get_mcu_info_via_swd() -> MCUInfo {
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

pub fn infer_core_from_chip_id(chip_id: &str) -> String {
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
