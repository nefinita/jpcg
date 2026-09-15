use serde::{Deserialize, Serialize};

use jpcg_api::CoefficientConfigDTO;

/// 等级换算系数（可配置载体）
///
/// 字段集合 = 等级常数（默认值单一来源：`jpcg_const::level_constant::CURRENT`，
/// 由 preset/cszj-exp-260908.toml 编译期固化；本结构自身不再内嵌数值字面量）。
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
        Self::from(jpcg_const::level_constant::CURRENT)
    }
}

impl CoefficientConfig {
    /// 正式服 130 级快照：历史数据回归用，不改变本分支一测默认。
    pub fn live_130() -> Self {
        Self::from(jpcg_const::level_constant::LIVE_130)
    }

    /// 50 级一测换算预设；仅替换分母，技能/血量与额外减伤规则需另行核实。
    pub fn cangsheng_first_test() -> Self {
        Self::from(jpcg_const::level_constant::CANGSHENG_FIRST_TEST)
    }
}

impl From<jpcg_const::level_constant::LevelConstant> for CoefficientConfig {
    fn from(c: jpcg_const::level_constant::LevelConstant) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_set::{hostilepile::HostilepileConfig, player::PlayerConfig};

    #[test]
    fn missing_coefficients_use_first_test_but_explicit_saved_values_survive()
    -> Result<(), serde_json::Error> {
        let missing: CoefficientConfig = serde_json::from_str("{}")?;
        assert_eq!(missing.huixin_xishu, 9512.91);
        let old: CoefficientConfig = serde_json::from_str(r#"{"huixin_xishu":197703.0}"#)?;
        assert_eq!(old.huixin_xishu, 197703.0);
        let dto: CoefficientConfigDTO = serde_json::from_str("{}")?;
        let from_dto = CoefficientConfig::from(&dto);
        assert_eq!(from_dto.huixin_xishu, 9512.91);
        // DTO 中 0 的 PVP 系数具有既有合法语义，不在本次改变。
        assert_eq!(from_dto.pvp_global_jianshang, 0.0);
        Ok(())
    }

    #[test]
    fn first_test_examples_use_new_defaults_and_keep_live_snapshot() {
        let coeff = CoefficientConfig::cangsheng_first_test();
        let player = PlayerConfig {
            huixin_dengji: 1319,
            ..Default::default()
        };
        let target = HostilepileConfig {
            waigong_fangyu: 412,
            neigong_fangyu: 412,
            huajin_dengji: 5148,
            ..Default::default()
        };
        assert!((player.guo_huixin_with(&coeff) - 0.138_653_68).abs() < 0.000_001);
        assert_eq!(target.guo_wfangyu_with(0, &coeff), 37);
        assert_eq!(target.guo_nfangyu_with(0, &coeff), 37);
        // 引擎既有额外 102/1024 化劲减伤保留；不是文章的裸属性百分比。
        assert_eq!(target.guo_huajin_with(&coeff), 512 + 102);
        assert_eq!(
            CoefficientConfig::default().huixin_xishu,
            coeff.huixin_xishu
        );
        assert_eq!(
            CoefficientConfig::from(jpcg_const::level_constant::LIVE_130).huixin_xishu,
            197703.0
        );
    }
}
