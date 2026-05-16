//! # 日志模块
//!
//! 提供标准化的文件日志记录功能，格式为 `[时间戳] [级别] 消息`，
//! 日志写入 ./logs/ 目录下每次启动生成的 .log 文件。

use chrono::Local;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Mutex<File>> = OnceLock::new();

/// 初始化日志文件，返回日志文件的绝对路径
///
/// 在 ./logs/ 目录创建以 YYYYMMDD-HHMMSS.log 命名的日志文件。
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

/// 写入一条标准化日志条目
///
/// 格式: `[YYYY-MM-DD HH:MM:SS] [LEVEL] message`
pub fn log(level: &str, message: &str) {
    if let Some(lock) = logger() {
        if let Ok(mut file) = lock.lock() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] [{}] {}", timestamp, level, message);
            let _ = file.flush();
        }
    }
}

/// 记录 INFO 级别日志
///
/// 用于正常的运行状态和操作信息。
pub fn info(message: &str) {
    log("INFO", message);
}

/// 记录 WARN 级别日志
///
/// 用于非致命的异常情况或需要注意的状态。
pub fn warn(message: &str) {
    log("WARN", message);
}

/// 记录 ERROR 级别日志
///
/// 用于错误或失败的操作。
#[allow(dead_code)]
pub fn error(message: &str) {
    log("ERROR", message);
}

/// 记录 DEBUG 级别日志
///
/// 用于开发调试信息。
#[allow(dead_code)]
pub fn debug(message: &str) {
    log("DEBUG", message);
}
