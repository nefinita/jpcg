use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct XinfaConfig {
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

impl XinfaConfig {
    pub fn new(
        xinfa_name: String,
        xinfa_nom: String,
        atk_up: f32,
        pofang_up: f32,
        huixin_up: f32,
    ) -> Self {
        Self { xinfa_name, xinfa_nom, atk_up, pofang_up, huixin_up }
    }

    pub fn default() -> Self {
        Self {
            xinfa_name: "莫问".to_string(),
            xinfa_nom: "根骨".to_string(),
            atk_up: 0.0,
            pofang_up: 0.0,
            huixin_up: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionInfo {
    pub level: u32,
    pub season: u32,
    pub modified: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct XinfaSummary {
    pub value: String,
    pub label: String,
    pub nom: String,
    pub version_label: Option<String>,
    pub version: VersionInfo,
}

pub fn get_xinfa_list() -> Vec<XinfaSummary> {
    crate::io::list_available_professions()
}
