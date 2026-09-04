// ============================================================================
// hostilepile — 目标（敌对）属性配置
// 包含目标的防御、御劲、化劲、减伤等属性及其计算公式。
// 公式参考剑网3 PVE/PVP 通用属性换算。
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::type_set::coefficient::CoefficientConfig;

/// 敌方（木桩/玩家目标）属性配置
/// 所有字段根据剑网3 属性压缩公式进行计算转换
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct HostilepileConfig {
    pub waigong_fangyu: u32, // 外功防御等级
    pub neigong_fangyu: u32, // 内功防御等级
    pub yujin_dengji: u32,   // 御劲等级（影响会心率和会心效果减免）
    pub huajin_dengji: u32,  // 化劲等级（影响伤害减免）
    pub jianshang_bili: u32, // 减伤比例（百分比，如 10 表示 10%）
    pub target_hp: u32,      // 目标血量（精确到个位），用于击杀概率计算
    /// 目标最大血量（追加真伤/击杀率用；0=未提供，回退 target_hp 满血模型）
    #[serde(default)]
    pub max_hp: u32,
    /// 目标当前血量（开局剩余；0=未提供，回退 target_hp 满血模型）
    #[serde(default)]
    pub current_hp: u32,
}

impl HostilepileConfig {
    /// 构造新的目标配置
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        waigong_fangyu: u32,
        neigong_fangyu: u32,
        yujin_dengji: u32,
        huajin_dengji: u32,
        jianshang_bili: u32,
    ) -> Self {
        Self {
            waigong_fangyu,
            neigong_fangyu,
            yujin_dengji,
            huajin_dengji,
            jianshang_bili,
            target_hp: 0,
            max_hp: 0,
            current_hp: 0,
        }
    }

    /// 计算外功防御系数（已扣除无视防御后）
    /// 公式: 防御×1024/(防御+防御系数)；防御系数 = 默认等级常数
    /// - `guo_wsfangyu`: 技能的无视防御值（以 1024 为基准）
    /// - 返回: 以 1024 为基准的防御系数（值越大减伤越多）
    pub fn guo_wfangyu(&self, guo_wsfangyu: u32) -> u32 {
        self.guo_wfangyu_with(guo_wsfangyu, &CoefficientConfig::default())
    }

    /// 计算内功防御系数（根骨/元气职业使用）
    /// 公式与外功防御相同，使用内功防御值计算
    pub fn guo_nfangyu(&self, guo_wsfangyu: u32) -> u32 {
        self.guo_nfangyu_with(guo_wsfangyu, &CoefficientConfig::default())
    }

    /// 计算化劲减伤系数
    /// 公式: 化劲/(化劲+化劲系数) + 102/1024；化劲系数 = 默认等级常数
    /// 返回值以 1024 为基准
    pub fn guo_huajin(&self) -> u32 {
        self.guo_huajin_with(&CoefficientConfig::default())
    }

    /// 计算御劲会心效果减免
    /// 公式: 御劲×1024/yuhui_xishu（御劲会伤减免分母，默认等级常数）
    /// 返回值以 1024 为基准
    pub fn guo_yujin_huixiao(&self) -> u32 {
        self.guo_yujin_huixiao_with(&CoefficientConfig::default())
    }

    /// 计算御劲会心率减免
    /// 公式: 御劲/yujin_xishu（御劲会心减免分母，默认等级常数）
    /// 返回值为小数（如 0.05 表示 5%）
    pub fn guo_yujin_huixin(&self) -> f32 {
        self.guo_yujin_huixin_with(&CoefficientConfig::default())
    }

    /// 使用可配置系数计算外功防御
    pub fn guo_wfangyu_with(&self, guo_wsfangyu: u32, coeff: &CoefficientConfig) -> u32 {
        let def = self.waigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0);
        (def * 1024.0 / (def + coeff.fangyu_xishu)) as u32
    }

    /// 使用可配置系数计算内功防御
    pub fn guo_nfangyu_with(&self, guo_wsfangyu: u32, coeff: &CoefficientConfig) -> u32 {
        let def = self.neigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0);
        (def * 1024.0 / (def + coeff.fangyu_xishu)) as u32
    }

    /// 使用可配置系数的化劲计算
    pub fn guo_huajin_with(&self, coeff: &CoefficientConfig) -> u32 {
        ((self.huajin_dengji as f32 / (self.huajin_dengji as f32 + coeff.huajin_xishu)
            + 102.0 / 1024.0)
            * 1024.0) as u32
    }

    /// 使用可配置系数的御劲会效减免
    /// 公式: 御劲×1024/yuhui_xishu（御劲 → 目标会心伤害减免）
    pub fn guo_yujin_huixiao_with(&self, coeff: &CoefficientConfig) -> u32 {
        (self.yujin_dengji as f32 * 1024.0 / coeff.yuhui_xishu) as u32
    }

    /// 使用可配置系数的御劲会心率减免
    /// 公式: 御劲/yujin_xishu（御劲 → 目标会心率减免）
    pub fn guo_yujin_huixin_with(&self, coeff: &CoefficientConfig) -> f32 {
        self.yujin_dengji as f32 / coeff.yujin_xishu
    }
}
