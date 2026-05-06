use colored::*;
use serde::Deserialize;
use std::fs;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ComponentMetadata {
    pub vendor_id: String,
    pub product_ids: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub flash_start_address: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub description: String,
    pub python_module: String,
    pub js_module: String,
    pub metadata: ComponentMetadata,
}

#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub components: Vec<ComponentInfo>,
}

pub struct PluginManager {
    pub manifest: PluginManifest,
}

impl PluginManager {
    pub fn load_from<P: AsRef<str>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let data = fs::read_to_string(path).ok()?;
        let manifest: PluginManifest = serde_yaml::from_str(&data).ok()?;
        Some(PluginManager { manifest })
    }

    pub fn list_components(&self) {
        println!("{}", "[插件组件]".magenta());
        for component in &self.manifest.components {
            println!(
                "  - {} ({}) : {}",
                component.name, component.id, component.description
            );
        }
    }

    pub fn find_component(&self, id: &str) -> Option<&ComponentInfo> {
        self.manifest.components.iter().find(|c| c.id == id)
    }

    pub fn default_stlink_component(&self) -> Option<&ComponentInfo> {
        self.find_component("stlink_v2")
    }
}
