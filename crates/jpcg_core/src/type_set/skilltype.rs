// ============================================================================
// skilltype — 技能数据结构
// 对应 TOML 中 [[skill]] 每一条的技能属性，
// 包含伤害系数、增伤/会心/会效加成、无视防御等战斗属性。
// ============================================================================

use serde::Deserialize;

/// 技能属性结构
/// 每个技能实例包含完整的伤害计算所需系数。
/// TOML 中以 [[skill]] 数组形式存在。
#[derive(Default, Deserialize, Clone)]
#[serde(default)]
pub struct Skilltype {
    pub skill_name: String,        // 技能名称
    pub skill_id: u32,             // 技能 ID
    pub sub_id: u32,               // 子技能 ID（同技能不同形态）
    pub group: u8,                 // 套路组编号（用于区分不同套路）
    pub weapon_request: u8,        // 所需武器类型（0=无要求, 1=单刀, 2=双刀...）
    pub design_effect: u8,         // 技能生效方式（1=直接伤害, 2=持续伤害Dot, 3=治疗...）
    pub kind_type: u8,             // 技能伤害类型（0=外功, 1=毒性内功, 2=混元内功, 3=阳性内功...）
    pub cast_mode: u8,             // 释放方式（0=单体, 1=群攻, 2=扇形, 3=矩形...）
    pub guaranteed_hit: bool,      // 必然命中（无视闪避/偏离）
    pub has_critical_strike: bool, // 无视会心限制（无质/必会心标签）
    pub effect_type: u8,           // 效果类型（0=有害, 1=有益）
    pub jihuoqixue: String,        // 激活该技能所需的奇穴名称
    pub base_damage1: u32,         // 基础伤害最小值
    pub base_damage2: u32,         // 基础伤害最大值
    pub atk_xishu: f32,            // 攻击力系数（攻击力转化为伤害的比例）
    pub watk_xishu: u32,           // 武器伤害系数（百分比，如 100 表示 100%）
    pub hit_up: u32,               // 增伤乘区（百分比，如 20 表示 20%）
    pub huixin_up: u32,            // 额外会心率（百分比）
    pub huixiao_up: u32,           // 额外会心效果（百分比）
    pub wushifangyu: u32,          // 无视防御（1024 制，如 512 无视 50%）
    pub wushihuajin: u32,          // 无视化劲
    pub wushijianshang: u32,       // 无视减伤
    pub zhenshishanghai: u32,      // 真实伤害（无视所有防御减免）
    pub dot_flag: u8,              // Dot 标签（0=非Dot, 1=Dot）
    pub dot_num: u8,               // Dot 总跳数
    pub dot_up: f32,               // Dot 递增系数（每跳递增比例）
}

impl Skilltype {
    /// 计算技能基础攻击（base_damage1 和 base_damage2 的平均值）
    pub fn base_atk(&self) -> u32 {
        (self.base_damage1 + self.base_damage2) / 2
    }
}
