//! # 日志模块
//!
//! 提供简单的文件日志记录功能，用于将运行时事件写入 ./logs/ 启动日志文件。

use chrono::Local;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init_logger() -> io::Result<String> {
    let cwd = env::current_dir()?;
    let mut logs_dir = cwd.join("logs");
    create_dir_all(&logs_dir)?;

    let filename = Local::now().format("%Y%m%d-%H%M%S").to_string() + ".log";
    logs_dir.push(filename);
    let file_path = logs_dir;

    let file = File::create(&file_path)?;
    LOGGER.set(Mutex::new(file)).ok();

    Ok(file_path.to_string_lossy().to_string())
}

fn logger() -> Option<&'static Mutex<File>> {
    LOGGER.get()
}

pub fn log(level: &str, message: &str) {
    if let Some(lock) = logger() {
        if let Ok(mut file) = lock.lock() {
            let _ = writeln!(file, "[{}] {}", level, message);
            let _ = file.flush();
        }
    }
}

pub fn info(message: &str) {
    log("INFO", message);
}

pub fn warn(message: &str) {
    log("WARN", message);
}

#[allow(dead_code)]
pub fn error(message: &str) {
    log("ERROR", message);
}

#[allow(dead_code)]
pub fn debug(message: &str) {
    log("DEBUG", message);
}
