use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for BuffConfig {
    fn default() -> Self {
        Self {
            base_atk_pct: 0.0,
            huixin_pct: 0.0,
            huixiao_pct: 0.0,
            pofang_pct: 0.0,
            wushi_fangyu_pct: 0.0,
            shanghai_pct: 0.0,
            mode_is_point: false,
        }
    }
}
