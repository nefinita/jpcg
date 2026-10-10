use serde::{Deserialize, Serialize};

use jpcg_api::CoefficientConfigDTO;

/// 等级换算系数（可配置载体）
///
/// 字段集合 = 等级常数。数值真源在 `data/values/*.toml`（运行期按值集加载，
/// 见 `store::values`）；本结构的 [`Default`] 固定指向**正式服 130 级**
/// （`jpcg_const::level_constant::LIVE_130`），作为历史基线与测试基线。
/// 实际运行时：调用方按选中值集传入 `base`（见 [`CoefficientConfig::from_dto`]），
/// 0/缺失字段回退到该值集的分母（分母为 0 非法）。
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
        // 基线固定为正式服 130 级（历史金标准/测试基线）；
        // 产品默认值集（体验服一测）由 store::values 在运行期提供。
        Self::from_level_constant(&jpcg_const::level_constant::LIVE_130)
    }
}

impl CoefficientConfig {
    /// 由指定值集的等级常数构造（0/缺失字段回退 `base`；pvp 全局减伤 0 = 无减伤，不回退）
    pub fn from_dto(
        dto: &CoefficientConfigDTO,
        base: &jpcg_const::level_constant::LevelConstant,
    ) -> Self {
        Self {
            pofang_xishu: nz(dto.pofang_xishu, base.pofang_xishu),
            huixin_xishu: nz(dto.huixin_xishu, base.huixin_xishu),
            huixiao_xishu: nz(dto.huixiao_xishu, base.huixiao_xishu),
            yujin_xishu: nz(dto.yujin_xishu, base.yujin_xishu),
            yuhui_xishu: nz(dto.yuhui_xishu, base.yuhui_xishu),
            huajin_xishu: nz(dto.huajin_xishu, base.huajin_xishu),
            fangyu_xishu: nz(dto.fangyu_xishu, base.fangyu_xishu),
            pvp_global_jianshang: dto.pvp_global_jianshang,
        }
    }

    /// 直接以某个值集的等级常数构造（不经过 DTO）
    pub fn from_level_constant(base: &jpcg_const::level_constant::LevelConstant) -> Self {
        Self {
            pofang_xishu: base.pofang_xishu,
            huixin_xishu: base.huixin_xishu,
            huixiao_xishu: base.huixiao_xishu,
            yujin_xishu: base.yujin_xishu,
            yuhui_xishu: base.yuhui_xishu,
            huajin_xishu: base.huajin_xishu,
            fangyu_xishu: base.fangyu_xishu,
            pvp_global_jianshang: base.pvp_global_jianshang,
        }
    }
}

impl From<&CoefficientConfigDTO> for CoefficientConfig {
    /// 兼容入口：以正式服 130 基线回退（运行期请用 [`CoefficientConfig::from_dto`] 传入选中值集）
    fn from(d: &CoefficientConfigDTO) -> Self {
        Self::from_dto(d, &jpcg_const::level_constant::LIVE_130)
    }
}

/// 0/缺失 → 真源默认（换算分母为 0 无意义；旧存档未含新字段时回退）
fn nz(v: f32, def: f32) -> f32 {
    if v > 0.0 { v } else { def }
}
