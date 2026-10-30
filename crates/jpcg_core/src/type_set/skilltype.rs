// ============================================================================
// skilltype — 技能数据结构
// 对应 TOML 中 [[skill]] 每一条的技能属性，
// 包含伤害系数、增伤/会心/会效加成、无视防御等战斗属性。
// ============================================================================

use serde::{Deserialize, Serialize};

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}
fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}
fn is_zero_f32(n: &f32) -> bool {
    *n == 0.0
}
fn is_false(b: &bool) -> bool {
    !b
}
// serde skip_serializing_if 要求 `&String` 签名，保持原样
#[allow(clippy::ptr_arg)]
fn is_string_empty(s: &String) -> bool {
    s.is_empty()
}

/// 技能属性结构
/// 每个技能实例包含完整的伤害计算所需系数。
/// TOML 中以 [[skill]] 数组形式存在。
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Skilltype {
    pub skill_name: String, // 技能名称
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub skill_id: u32, // 技能 ID
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub sub_id: u32, // 子技能 ID（同技能不同形态）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub group: u8, // 套路组编号（用于区分不同套路）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub weapon_request: u8, // 所需武器类型（0=无要求, 1=单刀, 2=双刀...）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub design_effect: u8, // 技能生效方式（1=直接伤害, 2=持续伤害Dot, 3=治疗...）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub kind_type: u8, // 技能伤害类型（0=外功, 1=毒性内功, 2=混元内功, 3=阳性内功...）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub cast_mode: u8, // 释放方式（0=单体, 1=群攻, 2=扇形, 3=矩形...）
    #[serde(skip_serializing_if = "is_false")]
    pub guaranteed_hit: bool, // 必然命中（无视闪避/偏离）
    #[serde(skip_serializing_if = "is_false")]
    pub has_critical_strike: bool, // 无质（伤害固定 = 期望 Q，含会心加权）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub effect_type: u8, // 效果类型（0=有害, 1=有益）
    #[serde(skip_serializing_if = "is_string_empty")]
    pub jihuoqixue: String, // 激活该技能所需的奇穴名称
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub base_damage1: u32, // 基础伤害最小值
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub base_damage2: u32, // 基础伤害最大值
    pub atk_xishu: f32,     // 攻击力系数（攻击力转化为伤害的比例）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub watk_xishu: u32, // 武器伤害系数（百分比，如 100 表示 100%）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub hit_up: u32, // 增伤乘区（百分比，如 20 表示 20%）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub huixin_up: u32, // 额外会心率（百分比）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub huixiao_up: u32, // 额外会心效果（百分比）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub wushifangyu: u32, // 无视防御（1024 制，如 512 无视 50%）
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub wushihuajin: u32, // 无视化劲
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub wushijianshang: u32, // 无视减伤
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub zhenshishanghai: u32, // 真实伤害（无视所有防御减免）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub lost_hp_zhenshishanghai: f32, // 追加真伤：目标已损失生命值 × 系数（如 0.06 = 6%/层），无视防御
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub dot_flag: u8, // Dot 标签（0=非Dot, 1=Dot）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub dot_interval: f32, // Dot 每跳间隔（秒）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub dot_duration: f32, // Dot 持续时长（秒）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub dot_up: f32, // Dot 递增系数（每跳递增比例，等比）
    // ---- #92 时序字段（旧数据缺省 = 瞬发 / 占普通 GCD / 无调息 / 无充能） ----
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub cast_time: f32, // 吟唱/读条时长（秒，0=瞬发）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub gcd_kind: u8, // GCD 类型（0=占普通 GCD，1=不占 GCD，2=固定 GCD 不受加速）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub cooldown: f32, // 调息时间（秒，0=无调息）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub charges: u8, // 充能层数（0 或 1 = 无充能；>1 = 最大层数）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub recharge: f32, // 充能恢复间隔（秒）
    #[serde(skip_serializing_if = "is_false")]
    pub cd_haste: bool, // 调息是否受加速影响
    #[serde(skip_serializing_if = "is_false")]
    pub channel: bool, // 引导技能（吟唱期间持续结算）
    #[serde(skip_serializing_if = "is_string_empty")]
    pub buff_id: String, // 仅提供 buff 的技能所施加的 buff id（空 = 不施加）
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub gcd_base: f32, // 该技能所在形态的基础公共 CD（秒；0 = 用门派默认，双形态门派按技能标注 1.5/1.0）
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub gcd_group: u8, // GCD 分组（0=默认组；双形态另一套技能用 1，两组 GCD 独立；不占 GCD 的技能由 gcd_kind=1 表达）
}

impl Skilltype {
    /// 计算技能基础攻击（base_damage1 和 base_damage2 的平均值）
    pub fn base_atk(&self) -> u32 {
        (self.base_damage1 + self.base_damage2) / 2
    }

    /// 是否只提供 buff 的技能（effect_type = 1 有益）
    pub fn is_buff_only(&self) -> bool {
        self.effect_type == 1
    }

    /// 是否占公共 CD（gcd_kind = 1 不占）
    pub fn occupies_gcd(&self) -> bool {
        self.gcd_kind != 1
    }

    /// 加速后的读条时长（秒；固定 GCD 技能不受加速影响，瞬时技能恒为 0）
    pub fn cast_time_with_haste(&self, haste_rate: f32) -> f32 {
        if self.gcd_kind == 2 {
            self.cast_time
        } else {
            crate::type_set::timing::quantize_secs(self.cast_time, haste_rate)
        }
    }

    /// 加速后可用的公共 CD 时长（秒；不占 GCD 返回 0；固定 GCD 不受加速）。
    /// `school_gcd_secs` 为门派默认 GCD；本技能标注了 `gcd_base` 时优先用技能的（双形态门派）。
    pub fn gcd_with_haste(&self, school_gcd_secs: f32, haste_rate: f32) -> f32 {
        let base = if self.gcd_base > 0.0 {
            self.gcd_base
        } else {
            school_gcd_secs.max(0.0)
        };
        match self.gcd_kind {
            1 => 0.0,
            2 => base,
            _ => crate::type_set::timing::quantize_gcd(base, haste_rate),
        }
    }

    /// 加速后的调息时间（秒；cd_haste=false 时原样返回）。
    /// 调息「直接吃百分比」：cooldown ÷ (1 + 加速率)，不按帧量化。
    pub fn cooldown_with_haste(&self, haste_rate: f32) -> f32 {
        if self.cd_haste && self.cooldown > 0.0 {
            self.cooldown / (1.0 + haste_rate.max(0.0))
        } else {
            self.cooldown
        }
    }

    /// 充能层数（0/1 视作 1 层，即无充能）
    pub fn max_charges(&self) -> u32 {
        if self.charges > 1 {
            self.charges as u32
        } else {
            1
        }
    }

    /// Dot 总跳数 = 持续时长 / 每跳间隔（均为 0 或数据非法时返回 0，即非 Dot）
    pub fn dot_jump_count(&self) -> u32 {
        if self.dot_interval > 0.0 && self.dot_duration >= self.dot_interval {
            (self.dot_duration / self.dot_interval).round() as u32
        } else {
            0
        }
    }
}
