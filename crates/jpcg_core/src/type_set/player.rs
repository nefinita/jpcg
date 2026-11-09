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
    #[serde(default)]
    pub jiasu_dengji: u32, // 加速等级（#92 时序引擎；0=无加速）
    /// 配装换算出的心法额外攻击。None 保留旧手填公式，Some 禁止二次转换主属性。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zhuanhua_gongji: Option<u32>,
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
            jiasu_dengji: 0,
            zhuanhua_gongji: None,
        }
    }

    /// 无增益攻击力：旧手填路径为基础攻击 + 主属性 × 心法系数；
    /// 配装路径为基础攻击 + 已换算心法额外攻击。武器伤害始终单独返回。
    pub fn atk(&self, shuxing_atk_up: f32) -> AtkConfig {
        self.atk_with_buff(shuxing_atk_up, 0.0)
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
        // 装备与基础属性合成后可能超过 u32::MAX / 1024；先扩宽整数乘法。
        // 原范围内乘积完全相同，后续 f32 换算及取整保持金标准语义。
        ((u64::from(self.pofang_dengji) * 1024) as f32 / coeff.pofang_xishu) as u32
    }

    /// 使用可配置系数的会效计算
    pub fn guo_huixinxiaoguo_with(&self, coeff: &CoefficientConfig) -> u32 {
        (self.huixin_xiaoguo as f32 * 1024.0 / coeff.huixiao_xishu) as u32
    }

    /// 使用可配置系数的会心率计算
    pub fn guo_huixin_with(&self, coeff: &CoefficientConfig) -> f32 {
        self.huixin_dengji as f32 / coeff.huixin_xishu
    }

    /// 计算加速率（小数；值集未启用加速系数时返回 0）
    pub fn guo_jiasu(&self) -> f32 {
        self.guo_jiasu_with(&CoefficientConfig::default())
    }

    /// 使用可配置系数计算加速率（小数）
    pub fn guo_jiasu_with(&self, coeff: &CoefficientConfig) -> f32 {
        crate::type_set::timing::jiasu_rate(self.jiasu_dengji, coeff.jiasu_xishu)
    }

    /// 计算最终攻击力（含心法加成 + 阵眼增益）
    pub fn atk_with_buff(&self, shuxing_atk_up: f32, buff_atk_pct: f32) -> AtkConfig {
        let base = match self.zhuanhua_gongji {
            // 新配装路径：基础攻击增益不放大心法额外攻击；在相加之前截断。
            // 历史 130 级参考模型将 base 与 extra 分开，证据见装备换算说明。
            Some(extra) => {
                let buffed =
                    (f64::from(self.jichu_gongji) * (1.0 + f64::from(buff_atk_pct) / 100.0)) as u32;
                buffed.saturating_add(extra)
            }
            // 旧手填路径保留 f32 运算与原取整位置，金标准数值不得漂移。
            None => {
                ((self.jichu_gongji as f32 + self.jichu_shuxing as f32 * shuxing_atk_up)
                    * (1.0 + buff_atk_pct / 100.0)) as u32
            }
        };
        AtkConfig {
            base,
            extra: self.wuqi_shanghai,
        }
    }
}

/// DTO → 领域模型的单一转换入口，单技能、连招、时序和配置存储共同使用。
impl From<jpcg_api::PlayerConfigDTO> for PlayerConfig {
    fn from(dto: jpcg_api::PlayerConfigDTO) -> Self {
        Self {
            jcsx: dto.jcsx,
            jichu_shuxing: dto.jichu_shuxing,
            jichu_gongji: dto.jichu_gongji,
            huixin_dengji: dto.huixin_dengji,
            huixin_xiaoguo: dto.huixin_xiaoguo,
            pofang_dengji: dto.pofang_dengji,
            wuqi_shanghai: dto.wuqi_shanghai,
            jiasu_dengji: dto.jiasu_dengji,
            zhuanhua_gongji: dto.zhuanhua_gongji,
            zuizhong_gongji: 0,
        }
    }
}

impl From<PlayerConfig> for jpcg_api::PlayerConfigDTO {
    fn from(player: PlayerConfig) -> Self {
        Self {
            jcsx: player.jcsx,
            jichu_shuxing: player.jichu_shuxing,
            jichu_gongji: player.jichu_gongji,
            huixin_dengji: player.huixin_dengji,
            huixin_xiaoguo: player.huixin_xiaoguo,
            pofang_dengji: player.pofang_dengji,
            wuqi_shanghai: player.wuqi_shanghai,
            jiasu_dengji: player.jiasu_dengji,
            zhuanhua_gongji: player.zhuanhua_gongji,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converted_attack_is_added_after_base_buff_and_replaces_main_conversion() {
        let mut player = PlayerConfig::new("根骨".into(), 100, 1001, 0, 0, 0, 999);
        assert_eq!(player.atk_with_buff(2.0, 12.5).total(), 1351);
        player.zhuanhua_gongji = Some(251);
        assert_eq!(player.atk_with_buff(2.0, 12.5).total(), 1377);
        // 心法旧系数无论为何值，都不能二次转换；武器伤害仍单独结算。
        assert_eq!(player.atk_with_buff(200.0, 12.5).total(), 1377);
        assert_eq!(player.atk(200.0).total(), 1252);
        assert_eq!(player.atk(200.0).atk_extra_show(), 999);
        player.zhuanhua_gongji = Some(0);
        assert_eq!(player.atk(200.0).total(), 1001);
    }

    #[test]
    fn derived_overcome_above_u32_multiplication_boundary_does_not_wrap() {
        // 4,000,000 装备破防 + 1,000,000 元气 × 0.3 = 4,300,000。
        // 此处用 1024 分母令预期值可直接手算，覆盖原 u32 × 1024 溢出点。
        let coeff = CoefficientConfig {
            pofang_xishu: 1024.0,
            ..Default::default()
        };
        for rating in [4_000_000, 4_194_303, 4_194_304, 4_300_000, u32::MAX] {
            let player = PlayerConfig {
                pofang_dengji: rating,
                ..Default::default()
            };
            assert_eq!(player.guo_pofang_with(&coeff), rating);
        }
    }

    #[test]
    fn converted_attack_saturates_instead_of_wrapping() {
        let player = PlayerConfig {
            jichu_gongji: u32::MAX,
            zhuanhua_gongji: Some(1),
            ..Default::default()
        };
        assert_eq!(player.atk_with_buff(0.0, 100.0).total(), u32::MAX);
    }

    #[test]
    fn dto_and_saved_config_roundtrip_preserve_converted_attack_and_haste() {
        let dto = jpcg_api::PlayerConfigDTO {
            jcsx: "根骨".into(),
            jichu_shuxing: 100,
            jichu_gongji: 1001,
            jiasu_dengji: 1234,
            zhuanhua_gongji: Some(251),
            ..Default::default()
        };
        let expected = serde_json::to_value(&dto).unwrap();
        let save = crate::store::SaveConfig {
            player: dto.into(),
            ..Default::default()
        };
        let text = toml::to_string(&save).unwrap();
        let restored: crate::store::SaveConfig = toml::from_str(&text).unwrap();
        let restored: jpcg_api::PlayerConfigDTO = restored.player.into();
        assert_eq!(serde_json::to_value(restored).unwrap(), expected);
    }

    #[test]
    fn old_json_and_toml_stay_in_legacy_mode() {
        let dto: jpcg_api::PlayerConfigDTO =
            serde_json::from_str(r#"{"jichu_shuxing":100,"jichu_gongji":1001}"#).unwrap();
        assert_eq!(dto.zhuanhua_gongji, None);
        let json = serde_json::to_value(&dto).unwrap();
        assert!(json.get("zhuanhua_gongji").is_none());
        let core = PlayerConfig::from(dto);
        let old = toml::to_string(&core).unwrap();
        assert!(!old.contains("zhuanhua_gongji"));
        let restored: PlayerConfig = toml::from_str(&old).unwrap();
        assert_eq!(restored.atk_with_buff(2.0, 12.5).total(), 1351);
    }
}
