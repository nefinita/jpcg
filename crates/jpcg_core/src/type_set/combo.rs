use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboStep {
    pub skill_id: u32,
    pub skill_name: String,
    pub overrides: Option<StepOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct StepOverride {
    pub base_damage_override: Option<f32>,
    pub atk_xishu_override: Option<f32>,
    pub jianshang_bili_override: Option<f32>,
    pub wushihuajin_override: Option<f32>,
    pub extra_atk_pct: Option<f32>,
    pub gain_override: Option<f32>,
    pub extra_crit_pct: Option<f32>,
    pub extra_crit_dmg_pct: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboPreset {
    pub name: String,
    pub steps: Vec<ComboStep>,
}
