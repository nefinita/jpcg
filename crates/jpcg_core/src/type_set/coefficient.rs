use serde::{Deserialize, Serialize};

use jpcg_api::CoefficientConfigDTO;

/// 等级换算系数（可配置载体）
///
/// 字段集合 = 等级常数（默认值单一来源：`jpcg_const::level_constant::CURRENT`，
/// 由 preset/level_constant.toml 编译期固化；本结构自身不再内嵌数值字面量）。
/// DTO 到本结构的转换用 [`From`]（0/缺失字段回退真源默认，分母为 0 非法）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoefficientConfig {
    pub pofang_xishu: f32,
    pub huixin_xishu: f32,
    pub huixiao_xishu: f32,
    /// 御劲 → 目标会心率减免（×1024 制分母）
    pub yujin_xishu: f32,
    /// 御劲 → 目标会心伤害减免（×1024 制分母）
    pub yuhui_xishu: f32,
    pub huajin_xishu: f32,
    pub fangyu_xishu: f32,
    pub pvp_global_jianshang: f32,
}

impl Default for CoefficientConfig {
    fn default() -> Self {
        let c = jpcg_const::level_constant::CURRENT;
        Self {
            pofang_xishu: c.pofang_xishu,
            huixin_xishu: c.huixin_xishu,
            huixiao_xishu: c.huixiao_xishu,
            yujin_xishu: c.yujin_xishu,
            yuhui_xishu: c.yuhui_xishu,
            huajin_xishu: c.huajin_xishu,
            fangyu_xishu: c.fangyu_xishu,
            pvp_global_jianshang: c.pvp_global_jianshang,
        }
    }
}

impl From<&CoefficientConfigDTO> for CoefficientConfig {
    fn from(d: &CoefficientConfigDTO) -> Self {
        let def = Self::default();
        Self {
            pofang_xishu: nz(d.pofang_xishu, def.pofang_xishu),
            huixin_xishu: nz(d.huixin_xishu, def.huixin_xishu),
            huixiao_xishu: nz(d.huixiao_xishu, def.huixiao_xishu),
            yujin_xishu: nz(d.yujin_xishu, def.yujin_xishu),
            yuhui_xishu: nz(d.yuhui_xishu, def.yuhui_xishu),
            huajin_xishu: nz(d.huajin_xishu, def.huajin_xishu),
            fangyu_xishu: nz(d.fangyu_xishu, def.fangyu_xishu),
            // pvp 全局减伤 0 = 无 PVP 减伤（合法语义），不做回退
            pvp_global_jianshang: d.pvp_global_jianshang,
        }
    }
}

/// 0/缺失 → 真源默认（换算分母为 0 无意义；旧存档未含新字段时回退）
fn nz(v: f32, def: f32) -> f32 {
    if v > 0.0 { v } else { def }
}
