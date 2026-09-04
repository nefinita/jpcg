// ============================================================================
// player — 玩家属性配置
// 包含玩家的基础属性、攻击、会心、破防、武器伤害等数据，
// 以及对应的属性换算公式（将等级/面板值转换为游戏内计算系数）。
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::type_set::coefficient::CoefficientConfig;

/// 玩家属性配置
/// 所有等级类字段（会心、破防等）通过相应的换算公式转为 1024 制系数
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerConfig {
    pub jcsx: String,         // 基础属性类型（根骨/力道/身法/元气）
    pub jichu_shuxing: u32,   // 基础属性值（如根骨点数）
    pub jichu_gongji: u32,    // 基础攻击力（不含武器伤害）
    pub huixin_dengji: u32,   // 会心等级
    pub huixin_xiaoguo: u32,  // 会心效果等级（会效）
    pub pofang_dengji: u32,   // 破防等级
    pub wuqi_shanghai: u32,   // 武器伤害（额外附加）
    pub zuizhong_gongji: u32, // 最终攻击力（外部计算填入，0=自动计算）
}

impl PlayerConfig {
    /// 构造完整的玩家配置
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        jcsx: String,
        jichu_shuxing: u32,
        jichu_gongji: u32,
        huixin_dengji: u32,
        huixin_xiaoguo: u32,
        pofang_dengji: u32,
        wuqi_shanghai: u32,
    ) -> Self {
        Self {
            jcsx,
            jichu_shuxing,
            jichu_gongji,
            huixin_dengji,
            huixin_xiaoguo,
            pofang_dengji,
            wuqi_shanghai,
            zuizhong_gongji: 0,
        }
    }

    /// 计算实际攻击力
    /// 公式: (基础攻击+基础属性) × (1 + 心法攻击加成)
    /// - `shuxing_atk_up`: 基础属性提供的攻击加成
    /// - 返回: AtkConfig，分离基础攻击和武器伤害
    pub fn atk(&self, shuxing_atk_up: f32) -> AtkConfig {
        let base = self.jichu_gongji as f32 + self.jichu_shuxing as f32 * shuxing_atk_up;
        AtkConfig {
            base: base as u32,
            extra: self.wuqi_shanghai,
        }
    }

    /// 计算破防系数（1024 制）
    /// 公式: 破防等级 × 1024 / 破防系数（默认等级常数）
    /// 返回值为 1024 为基准的系数
    pub fn guo_pofang(&self) -> u32 {
        self.guo_pofang_with(&CoefficientConfig::default())
    }

    /// 计算会心效果系数（1024 制）
    /// 公式: 会效等级 × 1024 / 会效系数（默认等级常数）
    pub fn guo_huixinxiaoguo(&self) -> u32 {
        self.guo_huixinxiaoguo_with(&CoefficientConfig::default())
    }

    /// 计算会心率（小数）
    /// 公式: 会心等级 / 会心系数（默认等级常数）
    /// 返回值为 0~1 之间的小数
    pub fn guo_huixin(&self) -> f32 {
        self.guo_huixin_with(&CoefficientConfig::default())
    }

    /// 使用可配置系数的破防计算
    pub fn guo_pofang_with(&self, coeff: &CoefficientConfig) -> u32 {
        ((self.pofang_dengji * 1024) as f32 / coeff.pofang_xishu) as u32
    }

    /// 使用可配置系数的会效计算
    pub fn guo_huixinxiaoguo_with(&self, coeff: &CoefficientConfig) -> u32 {
        (self.huixin_xiaoguo as f32 * 1024.0 / coeff.huixiao_xishu) as u32
    }

    /// 使用可配置系数的会心率计算
    pub fn guo_huixin_with(&self, coeff: &CoefficientConfig) -> f32 {
        self.huixin_dengji as f32 / coeff.huixin_xishu
    }

    /// 计算最终攻击力（含心法加成 + 阵眼增益）
    pub fn atk_with_buff(&self, shuxing_atk_up: f32, buff_atk_pct: f32) -> AtkConfig {
        let base = (self.jichu_gongji as f32 + self.jichu_shuxing as f32 * shuxing_atk_up)
            * (1.0 + buff_atk_pct / 100.0);
        AtkConfig {
            base: base as u32,
            extra: self.wuqi_shanghai,
        }
    }
}

// ============================================================================
// AtkConfig — 攻击力配置
// 将攻击力拆分为基础攻击和武器伤害两部分。
// ============================================================================

/// 攻击力配置（基础攻击 + 武器伤害）
pub struct AtkConfig {
    base: u32,  // 基础攻击力（基础攻击 + 属性转化）
    extra: u32, // 额外攻击力（武器伤害）
}

impl AtkConfig {
    /// 总攻击力（基础攻击 + 属性转化；不含武器伤害——武器伤害单独经 watk_xishu 参与技能伤害，避免双算）
    pub fn total(&self) -> u32 {
        self.base
    }

    /// 仅获取基础攻击部分
    pub fn atk_base_show(&self) -> u32 {
        self.base
    }

    /// 仅获取额外（武器）攻击部分
    pub fn atk_extra_show(&self) -> u32 {
        self.extra
    }
}
