use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct XinfaConfig {
    pub profession: String,
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

impl XinfaConfig {
    pub fn new(
        profession: String,
        xinfa_name: String,
        xinfa_nom: String,
        atk_up: f32,
        pofang_up: f32,
        huixin_up: f32,
    ) -> Self {
        Self {
            profession,
            xinfa_name,
            xinfa_nom,
            atk_up,
            pofang_up,
            huixin_up,
        }
    }

    pub fn default() -> Self {
        Self {
            profession: "mowen".to_string(),
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

impl VersionInfo {
    /// 数据版本号（紧凑格式，合法 semver）：等级.赛季.日期
    /// 例：level=130, season=3, modified=20260602 → "130.3.20260602"
    pub fn compact(&self) -> String {
        format!("{}.{}.{}", self.level, self.season, self.modified)
    }

    /// 数据版本号（带 v 前缀，供 update.toml / URL 使用）：v130.3.20260602
    pub fn compact_v(&self) -> String {
        format!("v{}.{}.{}", self.level, self.season, self.modified)
    }

    /// 日期展示：20260602 → "2026-06-02"
    fn modified_date_pretty(&self) -> String {
        let m = format!("{:08}", self.modified);
        if m.len() == 8 {
            format!("{}-{}-{}", &m[0..4], &m[4..6], &m[6..8])
        } else {
            m
        }
    }

    /// UI 美化展示：130级第3赛季 (2026-06-02)
    pub fn label(&self) -> Option<String> {
        if self.level > 0 {
            Some(format!(
                "{}级第{}赛季 ({})",
                self.level,
                self.season,
                self.modified_date_pretty()
            ))
        } else {
            None
        }
    }
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
    crate::store::list_available_professions()
}
