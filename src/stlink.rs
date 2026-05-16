//! # ST-Link 设备与 MCU 信息
//!
//! USB 设备检测、ST-Link 信息获取、SWD 读取 MCU 信息。

use crate::plugin::PluginManager;
use crate::utils::execute_command;
#[cfg(target_os = "linux")]
use std::fs;

/// MCU 信息
#[derive(Default)]
pub struct MCUInfo {
    pub chip_id: String,
    pub chip_name: String,
    pub flash_size: u32,
    pub core: String,
}

/// ST-Link 信息
#[derive(Default)]
pub struct STLinkInfo {
    pub version: String,
    pub serial: String,
    pub vid: u16,
    pub pid: u16,
}

// ── 辅助 ──

fn parse_hex(s: &str) -> Option<u16> {
    u16::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
}

fn default_metadata() -> Option<(u16, Vec<u16>)> {
    let mgr = PluginManager::load_from("plugins/manifest.yaml")?;
    let c = mgr.default_downloader()?;
    let vid = parse_hex(&c.metadata.vendor_id)?;
    let pids: Vec<u16> = c.metadata.product_ids.iter().filter_map(|p| parse_hex(p)).collect();
    Some((vid, pids))
}

// ── USB 检测 ──

pub fn detect_stlink_by_usb() -> bool {
    #[cfg(target_os = "linux")]
    {
        let (vid, pids) = default_metadata().unwrap_or((0x0483, vec![0x3748, 0x374B]));
        fs::read_dir("/sys/bus/usb/devices/")
            .ok().into_iter().flat_map(|d| d.flatten())
            .filter(|e| e.file_name().to_string_lossy().chars().next().map_or(false, |c| c != '.'))
            .filter_map(|e| {
                let p = e.path();
                let v = parse_hex(&fs::read_to_string(p.join("idVendor")).ok()?)?;
                let d = parse_hex(&fs::read_to_string(p.join("idProduct")).ok()?)?;
                Some((v, d))
            })
            .any(|(v, d)| v == vid && pids.contains(&d))
    }
    #[cfg(target_os = "windows")]
    {
        let o = execute_command("powershell", &["-Command",
            "Get-PnpDevice | Where-Object { $_.InstanceId -like '*USB*' -and ($_.DeviceID -like '*0483*' -or $_.DeviceID -like '*STLINK*') } | Select-Object -First 1"]);
        o.status == 0 && !o.stdout.trim().is_empty()
    }
    #[cfg(target_os = "macos")]
    {
        let o = execute_command("system_profiler", &["SPUSBDataType"]);
        o.status == 0 && o.stdout.contains("STMicroelectronics")
    }
}

// ── ST-Link 信息 ──

pub fn get_stlink_info() -> STLinkInfo {
    let mut info = STLinkInfo::default();

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if let Some(cli) = crate::utils::find_stlink_cli_tool() {
        #[cfg(target_os = "linux")]
        {
            let ver = execute_command(&cli, &["--version"]);
            if let Some(pos) = ver.stdout.find('v') {
                info.version = ver.stdout[pos..].split_whitespace().next().unwrap_or("Unknown").into();
            }
            let ser = execute_command(&cli, &["--serial"]);
            for line in ser.stdout.lines() {
                if let Some(val) = line.trim().strip_prefix("serial:") {
                    info.serial = val.trim().into(); break;
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            let ver = execute_command(&cli, &["-Version"]);
            info.version = if ver.status == 0 { ver.stdout.trim().into() } else { "Unknown".into() };
            info.serial = "N/A".into();
        }
    }

    if let Some((v, pids)) = default_metadata() {
        info.vid = v;
        info.pid = pids.first().copied().unwrap_or(0x3748);
    } else {
        info.vid = 0x0483;
        info.pid = 0x3748;
    }
    info
}

// ── MCU 信息 ──

pub fn parse_flash_size(text: &str) -> Option<u32> {
    let t = text.trim();
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    let val: u32 = digits.parse().ok()?;
    let suffix = t[digits.len()..].trim().to_ascii_lowercase();
    let mul = if suffix.starts_with("kib") || suffix.starts_with("kb") || suffix == "k" { 1024 }
    else if suffix.starts_with("mib") || suffix == "m" { 1024 * 1024 }
    else { 1 };
    Some(val * mul)
}

pub fn get_mcu_info_via_swd() -> MCUInfo {
    let mut info = MCUInfo::default();
    let cli = match crate::utils::find_stlink_cli_tool() { Some(p) => p, None => return info };

    // 方法 1: --probe
    let (probe_args, _) = if cfg!(target_os = "linux") {
        (vec!["--probe"], 0)
    } else {
        (vec!["-c", "SN=?", "-P"], 0)
    };
    let probe = execute_command(&cli, &probe_args.iter().map(|s| *s).collect::<Vec<_>>());

    if probe.status == 0 && probe.stdout.contains("chipid") {
        for line in probe.stdout.lines() {
            if let Some(v) = line.trim().strip_prefix("chipid:") { info.chip_id = v.trim().into(); }
            else if let Some(v) = line.trim().strip_prefix("flash:") { info.flash_size = parse_flash_size(v).unwrap_or(0); }
            else if let Some(v) = line.trim().strip_prefix("dev-type:") { info.chip_name = v.trim().into(); }
        }
        info.core = infer_core(&info.chip_id);
        return info;
    }

    // 方法 2: --flash + --chipid
    let flash_args: Vec<&str> = if cfg!(target_os = "linux") { vec!["--flash"] } else { vec!["-c", "SN=?", "-P"] };
    let flash = execute_command(&cli, &flash_args);
    if flash.status == 0 { info.flash_size = parse_flash_size(&flash.stdout).unwrap_or(0); }

    if info.flash_size > 0 {
        let cid_args: Vec<&str> = if cfg!(target_os = "linux") { vec!["--chipid"] } else { vec!["-c", "SN=?", "-P"] };
        let cid = execute_command(&cli, &cid_args);
        if cid.status == 0 { info.chip_id = cid.stdout.trim().into(); }
        (info.chip_name, info.core) = chip_family(&info.chip_id);
    }
    info
}

pub fn infer_core(id: &str) -> String {
    if id.contains("410") || id.contains("411") || id.contains("412") { "Cortex-M3".into() }
    else if id.contains("413") || id.contains("423") { "Cortex-M4".into() }
    else if id.contains("449") { "Cortex-M7".into() }
    else { "Cortex-Mx".into() }
}

fn chip_family(id: &str) -> (String, String) {
    if id.contains("410") || id.contains("411") || id.contains("412") { ("STM32F1".into(), "Cortex-M3".into()) }
    else if id.contains("413") || id.contains("423") { ("STM32F4".into(), "Cortex-M4".into()) }
    else if id.contains("449") { ("STM32F7".into(), "Cortex-M7".into()) }
    else { ("Unknown STM32".into(), "Cortex-Mx".into()) }
}
