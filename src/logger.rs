//! # 日志模块
//!
//! 文件日志，格式 `[时间] [级别] 消息`，写入 `./logs/YYYYMMDD-HHMMSS.log`。

use chrono::Local;
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init_logger() -> io::Result<String> {
    let mut dir = std::env::current_dir()?.join("logs");
    create_dir_all(&dir)?;
    dir.push(format!("{}.log", Local::now().format("%Y%m%d-%H%M%S")));
    let f = File::create(&dir)?;
    LOGGER.set(Mutex::new(f)).ok();
    Ok(dir.to_string_lossy().into_owned())
}

fn log(level: &str, msg: &str) {
    if let Some(lk) = LOGGER.get() {
        if let Ok(mut f) = lk.lock() {
            let _ = writeln!(f, "[{}] [{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), level, msg);
            let _ = f.flush();
        }
    }
}

pub fn info(msg: &str)  { log("INFO", msg); }
pub fn warn(msg: &str)  { log("WARN", msg); }
#[allow(dead_code)] pub fn error(msg: &str) { log("ERROR", msg); }
#[allow(dead_code)] pub fn debug(msg: &str) { log("DEBUG", msg); }
