use serde::{Deserialize, Serialize};

use super::skilltype::Skilltype;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboStep {
    pub skill_id: u32,
    /// 子技能 ID（区分同 skill_id 不同形态；旧存档缺失时回退 0）
    #[serde(default)]
    pub sub_id: u32,
    pub skill_name: String,
    /// 技能全量属性快照（连招计算用；保存/加载预设不丢失属性）。
    /// 旧存档（无快照）缺失时为 None，回退用 DTO 重建（属性不全）。
    #[serde(default)]
    pub skill_snapshot: Option<Skilltype>,
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
