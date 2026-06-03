// ============================================================================
// jcsx_set — 根骨/力道/身法/元气 属性映射
// 将游戏中的基础属性类型映射为对应的攻击/破防/会心加成。
// 当前为预留占位模块，核心逻辑尚未接入，统一默认使用 2 倍加成。
// ============================================================================

use std::collections::HashMap;

use crate::io::toml_input;
use crate::log::info;

/// 从 data/atk_config.toml 加载属性映射数据（预留）
fn data_load() {
    let content = toml_input("data/atk_config");
    info(format!("已加载属性映射配置:\n{}", content).as_str());
}

/// 基础属性映射配置
/// 将 "根骨"/"力道"/"身法"/"元气" 名称映射为对应的攻击/破防/会心加成值
pub struct JcsxConfig {
    pub jcsx_name: String,   // 基础属性名称
    pub jcsx_atk: f32,      // 攻击加成系数
    pub jcsx_pofang: f32,   // 破防加成系数
    pub jcsx_huixin: f32,   // 会心加成系数
}

/// 获取可用的基础属性映射表（尚未实现）
pub fn get_jcsx_list() -> HashMap<String, JcsxConfig> {
    todo!()
}
