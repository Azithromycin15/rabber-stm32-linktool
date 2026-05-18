//! # 国际化 (i18n) 模块
//!
//! 支持中文/英语双语言切换。
//!
//! ## 语言检测优先级
//! 1. `RABBER_LANG` 环境变量 (`zh`/`en`)
//! 2. 系统 locale (`LANG`/`LC_ALL`/`LC_MESSAGES`)
//! 3. 默认: 非中国地区使用英语
//!
//! ## 使用方式
//! ```
//! use crate::t;
//! println!("{}", t!("你好", "Hello"));
//! println!("{}", format!("{} {}", t!("版本", "Version"), ver));
//! ```

use std::sync::OnceLock;

/// 语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

static LANG: OnceLock<Lang> = OnceLock::new();

/// 初始化语言设置（应在 `main()` 早期调用）
pub fn init_lang() {
    let lang = detect_lang();
    LANG.set(lang).ok();
    match lang {
        Lang::Zh => eprintln!("[i18n] 语言: 中文"),
        Lang::En => eprintln!("[i18n] Language: English"),
    }
}

/// 获取当前语言
pub fn lang() -> Lang {
    *LANG.get().unwrap_or(&Lang::En)
}

/// 检测系统语言
fn detect_lang() -> Lang {
    // 1. 环境变量 RABBER_LANG 优先
    if let Ok(val) = std::env::var("RABBER_LANG") {
        let low = val.to_lowercase();
        if low.starts_with("zh") || low.starts_with("cn") {
            return Lang::Zh;
        }
        if low.starts_with("en") {
            return Lang::En;
        }
    }

    // 2. 检查系统 locale
    for var in &["LANG", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(val) = std::env::var(var) {
            let low = val.to_lowercase();
            if low.starts_with("zh") {
                return Lang::Zh;
            }
            if low.starts_with("en") {
                return Lang::En;
            }
        }
    }

    // 3. 默认英语
    Lang::En
}

/// 国际化字符串宏
///
/// `t!("中文", "English")` 根据当前语言返回对应字符串。
/// 可用于 `format!()` 等需要 `&str` 的场合。
#[macro_export]
macro_rules! t {
    ($zh:expr, $en:expr) => {{
        match $crate::i18n::lang() {
            $crate::i18n::Lang::Zh => $zh,
            $crate::i18n::Lang::En => $en,
        }
    }};
}

/// 格式化国际化字符串
///
/// `tfmt!("你好, {}!", "Hello, {}!", name)`
#[macro_export]
macro_rules! tfmt {
    ($zh:literal, $en:literal $(, $arg:expr)+ $(,)?) => {{
        match $crate::i18n::lang() {
            $crate::i18n::Lang::Zh => format!($zh $(, $arg)+),
            $crate::i18n::Lang::En => format!($en $(, $arg)+),
        }
    }};
}
