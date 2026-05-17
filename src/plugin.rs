//! # 插件管理
//!
//! 加载插件清单、组件查找、命令验证。

use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentMetadata {
    pub vendor_id: String,
    pub product_ids: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub flash_start_address: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentAction {
    pub name: String,
    pub description: String,
    pub args: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentInfo {
    pub id: String,
    pub plugin_name: String,
    pub command: String,
    pub name: String,
    pub component_type: String,
    pub description: String,
    pub python_module: String,
    pub js_module: String,
    pub metadata: ComponentMetadata,
    pub actions: Option<Vec<ComponentAction>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub components: Vec<ComponentInfo>,
}

pub struct PluginManager {
    manifest: PluginManifest,
}

impl PluginManager {
    pub fn load_from(path: &str) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        Some(Self { manifest: serde_yaml::from_str(&data).ok()? })
    }

    pub fn probe_and_generate_manifest(plugins_dir: &Path, manifest_path: &Path) -> Option<Self> {
        let components: Vec<ComponentInfo> = fs::read_dir(plugins_dir).ok()?
            .flatten().map(|e| e.path()).filter(|p| p.is_dir())
            .filter_map(|d| {
                fs::File::open(d.join("js").join("component.json")).ok()
                    .and_then(|f| serde_json::from_reader(f).ok())
            }).collect();
        let m = PluginManifest { components };
        if let Some(p) = manifest_path.parent() { let _ = fs::create_dir_all(p); }
        let _ = fs::write(manifest_path, serde_yaml::to_string(&m).unwrap_or_default());
        Some(Self { manifest: m })
    }

    pub fn count(&self) -> usize { self.manifest.components.len() }
    pub fn ready(&self) -> bool { !self.manifest.components.is_empty() }

    pub fn list(&self) {
        println!("{}", "[插件]".magenta());
        for c in &self.manifest.components {
            println!("  - {} ({}) : {}", c.name, c.id, c.description);
        }
    }

    pub fn find(&self, id: &str) -> Option<&ComponentInfo> {
        self.manifest.components.iter().find(|c| c.id == id)
    }

    pub fn download_components(&self) -> Vec<&ComponentInfo> {
        self.manifest.components.iter()
            .filter(|c| Path::new(&c.python_module).file_name().map_or(false, |n| n == "downloader.py"))
            .collect()
    }

    pub fn default_downloader(&self) -> Option<&ComponentInfo> {
        self.find("stlink_v2").or_else(|| self.find("cmsis_dap"))
            .or_else(|| self.manifest.components.iter().find(|c| c.component_type == "debugger"))
    }

    pub fn actions(&self, id: &str) -> Option<&[ComponentAction]> {
        self.find(id).and_then(|c| c.actions.as_deref())
    }

    pub fn has_action(&self, id: &str, action: &str) -> bool {
        self.actions(id).map_or(false, |a| a.iter().any(|x| x.name == action))
    }

    pub fn help(&self, id: &str) {
        match self.find(id) {
            Some(c) => {
                println!("{}", format!("[插件 {}]", c.name).cyan());
                match c.actions.as_ref().filter(|a| !a.is_empty()) {
                    Some(actions) => for a in actions {
                        println!("  {} {} {}", c.id, a.name, a.args.as_deref().unwrap_or(""));
                        println!("      {}", a.description);
                    },
                    None => println!("  无可用命令"),
                }
            }
            None => println!("{}", format!("未知插件: {}", id).red()),
        }
    }

    /// 列出所有可用插件及其命令用法
    pub fn help_all_plugins(&self) {
        if self.manifest.components.is_empty() {
            println!("{}", "无可用插件".yellow());
            return;
        }
        for c in &self.manifest.components {
            println!("{}", format!("[{}] {} ({})", c.command, c.name, c.id).cyan());
            println!("  {}", c.description);
            match c.actions.as_ref().filter(|a| !a.is_empty()) {
                Some(actions) => {
                    for a in actions {
                        println!("  {} {} {}", c.id, a.name, a.args.as_deref().unwrap_or(""));
                        println!("      {}", a.description);
                    }
                }
                None => println!("  无可用命令"),
            }
            println!();
        }
    }
}
