use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboStep {
    pub skill_id: u32,
    pub skill_name: String,
    pub overrides: Option<StepOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for StepOverride {
    fn default() -> Self {
        Self {
            base_damage_override: None,
            atk_xishu_override: None,
            jianshang_bili_override: None,
            wushihuajin_override: None,
            extra_atk_pct: None,
            gain_override: None,
            extra_crit_pct: None,
            extra_crit_dmg_pct: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboPreset {
    pub name: String,
    pub steps: Vec<ComboStep>,
}
