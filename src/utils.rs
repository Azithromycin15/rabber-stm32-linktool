use std::path::Path;
use std::process::{Command, Stdio};
use which::which;

pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
}

pub fn is_root() -> bool {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        if output.status.success() {
            let uid = String::from_utf8_lossy(&output.stdout);
            return uid.trim() == "0";
        }
    }
    false
}

pub fn execute_command(cmd: &str, args: &[&str]) -> CommandResult {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

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

pub fn find_tool(name: &str, possible_paths: &[&str]) -> Option<String> {
    for path in possible_paths {
        if Path::new(path).is_file() {
            return Some(path.to_string());
        }
    }

    if let Ok(path) = which(name) {
        return Some(path.to_string_lossy().to_string());
    }

    None
}

pub fn find_stlink_cli_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-info",
        "/usr/local/bin/st-info",
        "/bin/st-info",
        "/usr/bin/stlink-info",
        "/usr/local/bin/stlink-info",
    ];
    find_tool("st-info", &possible_paths)
}

pub fn find_stlink_programmer_tool() -> Option<String> {
    let possible_paths = [
        "/usr/bin/st-flash",
        "/usr/local/bin/st-flash",
        "/bin/st-flash",
        "/usr/bin/stlink-flash",
        "/usr/local/bin/stlink-flash",
    ];
    find_tool("st-flash", &possible_paths)
}

pub fn check_stlink_tools_installed() -> bool {
    find_stlink_cli_tool().is_some() && find_stlink_programmer_tool().is_some()
}
