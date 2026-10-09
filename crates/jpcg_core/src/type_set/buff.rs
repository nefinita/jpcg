use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BuffConfig {
    pub base_atk_pct: f32,
    pub huixin_pct: f32,
    pub huixiao_pct: f32,
    pub pofang_pct: f32,
    pub wushi_fangyu_pct: f32,
    pub shanghai_pct: f32,
    pub mode_is_point: bool,
}
