//! # CLI 参数定义
//!
//! 这个模块定义了命令行接口的参数结构。

use clap::Parser;

/// 命令行参数结构
///
/// 当前版本中主要用于显示版本信息和帮助。
#[derive(Parser)]
#[command(author, version, about = "由Rust构建的 ST-Link V2 工具", long_about = None)]
pub struct Args {}
