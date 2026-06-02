use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoefficientConfig {
    pub pofang_xishu: f32,
    pub huixin_xishu: f32,
    pub huixiao_xishu: f32,
    pub huajin_xishu: f32,
    pub fangyu_xishu: f32,
    pub pvp_global_jianshang: f32,
}

impl Default for CoefficientConfig {
    fn default() -> Self {
        Self {
            pofang_xishu: 225957.6,
            huixin_xishu: 197703.0,
            huixiao_xishu: 72844.2,
            huajin_xishu: 30115.8,
            fangyu_xishu: 126007.2,
            pvp_global_jianshang: 0.9,
        }
    }
}
