// ============================================================================
// xinfa — 心法配置
// 包含心法的名称、根骨/元气属性、以及心法提供的攻击/破防/会心加成。
// TOML 中以 [xinfa] 节存在。
// ============================================================================

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 心法基础配置
/// 对应 TOML 配置文件中 [xinfa] 节的字段
#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct XinfaConfig {
    pub xinfa_name: String,   // 心法名称（如 "莫问"、"傲血"）
    pub xinfa_nom: String,    // 基础属性类型（根骨/元气/力道/身法）
    pub atk_up: f32,          // 攻击力百分比加成（如 0.05 表示 5%）
    pub pofang_up: f32,       // 破防百分比加成
    pub huixin_up: f32,       // 会心百分比加成
}

impl XinfaConfig {
    /// 构造完整的心法配置
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        xinfa_name: String,
        xinfa_nom: String,
        atk_up: f32,
        pofang_up: f32,
        huixin_up: f32,
    ) -> Self {
        Self {
            xinfa_name,
            xinfa_nom,
            atk_up,
            pofang_up,
            huixin_up,
        }
    }

    /// 默认心法配置（莫问·根骨）
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

/// 获取所有可用的心法配置列表（尚未实现）
pub fn get_xinfa_list() -> HashMap<String, XinfaConfig> {
    todo!()
}
