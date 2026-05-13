//! # 插件管理模块
//!
//! 这个模块负责加载和管理插件清单，包括组件信息的解析、
//! 组件查找和命令验证功能。

use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 组件元数据结构
///
/// 包含组件的硬件信息，如供应商 ID、产品 ID 等。
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentMetadata {
    pub vendor_id: String,
    pub product_ids: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub flash_start_address: Option<String>,
}

/// 组件动作结构
///
/// 定义组件支持的命令及其描述。
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentAction {
    pub name: String,
    pub description: String,
    pub args: Option<String>,
}

/// 组件信息结构
///
/// 包含组件的完整信息，包括 ID、名称、类型和支持的动作。
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

/// 插件清单结构
///
/// 包含所有可用组件的列表。
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub components: Vec<ComponentInfo>,
}

/// 插件管理器
///
/// 负责加载插件清单并提供组件查询功能。
pub struct PluginManager {
    pub manifest: PluginManifest,
}

impl PluginManager {
    /// 从文件加载插件清单
    ///
    /// 读取 YAML 格式的插件清单文件并解析为 PluginManifest。
    pub fn load_from<P: AsRef<str>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let data = fs::read_to_string(path).ok()?;
        let manifest: PluginManifest = serde_yaml::from_str(&data).ok()?;
        Some(PluginManager { manifest })
    }

    /// 探测并生成插件清单
    ///
    /// 扫描 plugins 目录，读取每个组件的 component.json 文件并生成 manifest.yaml。
    pub fn probe_and_generate_manifest<P: AsRef<Path>>(plugins_dir: P, manifest_path: P) -> Option<Self> {
        let plugins_dir = plugins_dir.as_ref();
        let manifest_path = manifest_path.as_ref();

        let components = fs::read_dir(plugins_dir)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .filter_map(|path| {
                let component_json = path.join("js").join("component.json");
                fs::File::open(&component_json)
                    .ok()
                    .and_then(|file| serde_json::from_reader(file).ok())
            })
            .collect::<Vec<_>>();

        let manifest = PluginManifest { components };
        if let Some(parent_dir) = manifest_path.parent() {
            let _ = fs::create_dir_all(parent_dir);
        }
        let _ = fs::write(manifest_path, serde_yaml::to_string(&manifest).unwrap_or_default());
        Some(PluginManager { manifest })
    }

    /// 组件总数
    pub fn count_components(&self) -> usize {
        self.manifest.components.len()
    }

    /// 插件清单是否具备可用组件
    pub fn is_ready(&self) -> bool {
        !self.manifest.components.is_empty()
    }

    /// 列出所有组件
    ///
    /// 打印所有已加载组件的名称、ID 和描述。
    pub fn list_components(&self) {
        println!("{}", "[插件组件]".magenta());
        for component in &self.manifest.components {
            println!(
                "  - {} ({}) : {}",
                component.name, component.id, component.description
            );
        }
    }

    /// 查找组件
    ///
    /// 根据组件 ID 查找对应的组件信息。
    pub fn find_component(&self, id: &str) -> Option<&ComponentInfo> {
        self.manifest.components.iter().find(|c| c.id == id)
    }

    /// 获取默认 ST-Link 组件
    ///
    /// 返回默认的 ST-Link V2 组件信息。
    pub fn default_stlink_component(&self) -> Option<&ComponentInfo> {
        self.find_component("stlink_v2")
    }

    /// 获取组件动作列表
    ///
    /// 返回指定组件支持的所有动作。
    pub fn component_actions(&self, id: &str) -> Option<&[ComponentAction]> {
        self.find_component(id)
            .and_then(|component| component.actions.as_deref())
    }

    /// 检查组件是否支持指定动作
    ///
    /// 验证组件是否支持给定的动作名称。
    pub fn has_action(&self, id: &str, action_name: &str) -> bool {
        self.component_actions(id)
            .map_or(false, |actions| actions.iter().any(|action| action.name == action_name))
    }

    /// 打印组件帮助信息
    ///
    /// 显示指定组件的所有可用命令及其描述。
    pub fn print_component_help(&self, id: &str) {
        if let Some(component) = self.find_component(id) {
            println!("{}", format!("[插件 {} 的可用命令]", component.name).cyan());
            if let Some(actions) = component.actions.as_ref() {
                for action in actions {
                    let args = action.args.as_deref().unwrap_or("");
                    println!("  {} {} {}", component.id, action.name, args);
                    println!("      {}", action.description);
                }
            } else {
                println!("  无可用命令定义。");
            }
        } else {
            println!("{}", format!("未知插件: {}", id).red());
        }
    }
}
