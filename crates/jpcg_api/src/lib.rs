// ============================================================================
// jpcg_api — JPCG 纯类型契约
// 提供跨进程共享的 DTO 类型（Tauri IPC / FFI JSON 双端共用）。
// 仅含序列化类型，不包含任何计算逻辑，不依赖 jpcg_core。
// 双端通过本 crate 保证 JSON 契约单一来源，防止漂移。
// ============================================================================

pub mod attributes;
pub use attributes::{
    AttributeConversion, AttributeDamageKind, AttributeRatio, AttributeRules, AttributeTarget,
    BasicAttribute,
};

// ============ 玩家/目标/心法/增益/系数 配置 DTO ============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct XinfaSummaryDTO {
    pub value: String,
    pub label: String,
    pub nom: String,
    pub version_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerConfigDTO {
    pub jcsx: String,
    pub jichu_shuxing: u32,
    pub jichu_gongji: u32,
    pub huixin_dengji: u32,
    pub huixin_xiaoguo: u32,
    pub pofang_dengji: u32,
    pub wuqi_shanghai: u32,
    /// 加速等级（#92 时序引擎；0=无加速）
    #[serde(default)]
    pub jiasu_dengji: u32,
    /// 已换算的心法额外攻击；Some(0) 也禁用旧主属性转换。
    /// 基础攻击增益只作用于 jichu_gongji，此项在其取整后相加。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zhuanhua_gongji: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HostileConfigDTO {
    pub waigong_fangyu: u32,
    pub neigong_fangyu: u32,
    pub yujin_dengji: u32,
    pub huajin_dengji: u32,
    pub jianshang_bili: u32,
    pub target_hp: u32,
    /// 目标最大血量（追加真伤/击杀率用；0=未提供，回退 target_hp 满血模型）
    #[serde(default)]
    pub max_hp: u32,
    /// 目标当前血量（开局剩余；0=未提供，回退 target_hp 满血模型）
    #[serde(default)]
    pub current_hp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BuffConfigDTO {
    pub base_atk_pct: f32,
    pub huixin_pct: f32,
    pub huixiao_pct: f32,
    pub pofang_pct: f32,
    pub wushi_fangyu_pct: f32,
    pub shanghai_pct: f32,
    pub mode_is_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CoefficientConfigDTO {
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
    /// 加速等级系数（加速率 = 加速等级 / 该系数 / 100；0 = 不启用加速换算）
    #[serde(default)]
    pub jiasu_xishu: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct XinfaConfigDTO {
    pub profession: String,
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
    /// 门派基础公共 CD（秒；0/缺失 = 1.5；特殊门派 1.0，双形态门派另有 per-skill gcd_base）
    #[serde(default)]
    pub gcd_base: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa_config: XinfaConfigDTO,
    pub buff: BuffConfigDTO,
    pub coefficient: CoefficientConfigDTO,
    /// 数值集 id（None = 使用默认值集，见 data/values/index.toml 的 default）
    #[serde(default)]
    pub value_set: Option<String>,
}

/// 属性面板结果；百分数单位（10 表示 10%），不含技能专属增益。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelStatsDTO {
    pub huixin_pct: f32,
    pub pofang_pct: f32,
    pub jianshang_pct: u32,
}

/// 完整配置数据（对应 saved_config.toml 的加载结果）
/// 字段形状与 CalculateRequest 完全一致（player/hostile/xinfa_config/buff/coefficient）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigDataDTO {
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa_config: XinfaConfigDTO,
    pub buff: BuffConfigDTO,
    pub coefficient: CoefficientConfigDTO,
    /// 数值集 id（持久化选择；None = 默认值集）
    pub value_set: Option<String>,
}

/// 数值集信息（对应 data/values/index.toml 的条目）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ValueSetDTO {
    /// 唯一 id（如 cszj-exp-260908 / live-130 / builtin-*）
    pub id: String,
    /// 界面显示名（如「体验服·苍生铸世一测」）
    pub name: String,
    /// 等级（仅标识）
    pub level: u32,
    /// 是否为当前默认值集
    pub is_default: bool,
    /// 来源："data"（本地 data/values）或 "builtin"（内置兜底）
    pub source: String,
    /// 索引已列出且对应快照可读取、可解析且校验通过
    pub available: bool,
    /// 快照可用时的换算常数（用于前端 seed「系数设置」；不可用时为 None）
    #[serde(default)]
    pub coefficient: Option<CoefficientConfigDTO>,
}

// ============ 计算/连招结果 DTO ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResultDTO {
    pub skill_name: String,
    pub y: u32,
    pub b: u32,
    pub i: u32,
    pub n: u32,
    pub h: u32,
    pub q: u32,
    /// Dot 每跳期望伤害（非 Dot 技能为空；q 为各跳之和）
    #[serde(default)]
    pub dot_jumps: Vec<u32>,
    /// 无质（伤害固定 = 期望 Q，含会心加权）
    #[serde(default)]
    pub has_critical_strike: bool,
    /// 真实伤害（数据源 custom_damage_base 标签，无视防御减免）
    #[serde(default)]
    pub zhenshishanghai: u32,
    /// 追加真伤系数（已损失生命值 × 系数，连招中动态结算，单技能面板满血为 0）
    #[serde(default)]
    pub lost_hp_zhenshishanghai: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillPoolItemDTO {
    pub skill_name: String,
    pub skill_id: u32,
    /// 子技能 ID（同 skill_id 不同形态，如引窍·0点任脉 ~ 引窍·100点任脉）
    #[serde(default)]
    pub sub_id: u32,
    pub base_damage1: u32,
    pub base_damage2: u32,
    pub atk_xishu: f32,
    pub watk_xishu: u32,
    pub hit_up: u32,
    pub huixin_up: u32,
    pub huixiao_up: u32,
    pub wushifangyu: u32,
    pub wushihuajin: u32,
    pub dot_flag: u8,
    /// Dot 每跳间隔（秒）
    #[serde(default)]
    pub dot_interval: f32,
    /// Dot 持续时长（秒）
    #[serde(default)]
    pub dot_duration: f32,
    /// Dot 递增系数（每跳递增比例，等比）
    #[serde(default)]
    pub dot_up: f32,
    /// 无视减伤
    #[serde(default)]
    pub wushijianshang: u32,
    /// 真实伤害（无视所有防御减免）
    #[serde(default)]
    pub zhenshishanghai: u32,
    /// 无质（伤害固定 = 期望 Q，含会心加权）
    #[serde(default)]
    pub has_critical_strike: bool,
    /// 追加真伤系数（已损失生命值 × 系数，0=无；连招中动态结算）
    #[serde(default)]
    pub lost_hp_zhenshishanghai: f32,
    // ---- #92 时序字段（与 SkillEditorItemDTO 同义） ----
    /// 吟唱/读条时长（秒，0=瞬发）
    #[serde(default)]
    pub cast_time: f32,
    /// GCD 类型（0=占普通 GCD，1=不占 GCD，2=固定 GCD 不受加速）
    #[serde(default)]
    pub gcd_kind: u8,
    /// 调息时间（秒，0=无调息）
    #[serde(default)]
    pub cooldown: f32,
    /// 充能层数（0 = 普通冷却；>=1 = 最大充能次数，使用 recharge 恢复）
    #[serde(default)]
    pub charges: u8,
    /// 充能恢复间隔（秒）
    #[serde(default)]
    pub recharge: f32,
    /// 调息是否受加速影响
    #[serde(default)]
    pub cd_haste: bool,
    /// 引导技能
    #[serde(default)]
    pub channel: bool,
    /// 仅提供 buff 的技能所施加的 buff id（空 = 不施加）
    #[serde(default)]
    pub buff_id: String,
    /// 该技能所在形态的基础公共 CD（秒；0 = 用门派默认）
    #[serde(default)]
    pub gcd_base: f32,
    /// GCD 分组（0 = 默认组；双形态门派另一套技能用 1，两组 GCD 互相独立）
    #[serde(default)]
    pub gcd_group: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StepOverrideDTO {
    pub base_damage_override: Option<f32>,
    pub atk_xishu_override: Option<f32>,
    pub jianshang_bili_override: Option<f32>,
    pub wushihuajin_override: Option<f32>,
    pub extra_atk_pct: Option<f32>,
    pub gain_override: Option<f32>,
    pub extra_crit_pct: Option<f32>,
    pub extra_crit_dmg_pct: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComboStepDTO {
    pub skill: SkillPoolItemDTO,
    pub overrides: Option<StepOverrideDTO>,
}

/// 标记位于第 position 个技能之前（等于技能数时位于末尾）；同位置按数组顺序。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboFlagDTO {
    pub position: usize,
    pub start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComboPresetDTO {
    pub flags: Vec<ComboFlagDTO>,
    pub name: String,
    pub steps: Vec<ComboStepDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboStepResultDTO {
    pub skill_name: String,
    pub g_damage: u32,
    pub h_damage: u32,
    pub q_damage: u32,
    pub crit_rate: f32,
    pub cumulative_mean_wan: f64,
    pub kill_prob: f64,
    /// Dot 每跳期望伤害（非 Dot 技能为空；q_damage 为各跳之和）
    #[serde(default)]
    pub dot_jumps: Vec<u32>,
    /// 无质（伤害固定 = 期望 Q，含会心加权）
    #[serde(default)]
    pub has_critical_strike: bool,
    /// 真实伤害（数据源 custom_damage_base 标签，无视防御减免）
    #[serde(default)]
    pub zhenshishanghai: u32,
    /// 本步追加真伤（已损失生命值 × 系数，无视防御，确定性只加期望）
    #[serde(default)]
    pub lost_hp_zhenshi_damage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboResultDTO {
    #[serde(default)]
    pub total_lost_hp_damage: f64,
    pub steps: Vec<ComboStepResultDTO>,
    pub total_expected_damage_wan: f64,
    pub final_kill_prob: f64,
    pub kill_prob_curve: Vec<(usize, f64)>,
}

// ============ 自动求导 DTO ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDerivativeDTO {
    pub skill_name: String,
    pub derivative: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativeEntryDTO {
    pub attr_name: String,
    pub attr_id: String,
    pub current_value: f32,
    pub total_derivative: f32,
    pub per_skill: Vec<SkillDerivativeDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritVsPofangDTO {
    pub better: String,
    pub huixin_total: f32,
    pub pofang_total: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAttrDTO {
    pub attr_name: String,
    pub attr_id: String,
    pub total_derivative: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeRecommendationDTO {
    pub crit_vs_pofang: CritVsPofangDTO,
    pub top3: Vec<TopAttrDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativesOutputDTO {
    pub derivatives: Vec<DerivativeEntryDTO>,
    pub recommendation: OptimizeRecommendationDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillEditorItemDTO {
    pub skill_name: String,
    pub skill_id: u32,
    pub sub_id: u32,
    pub group: u8,
    pub weapon_request: u8,
    pub design_effect: u8,
    pub kind_type: u8,
    pub cast_mode: u8,
    pub guaranteed_hit: bool,
    pub has_critical_strike: bool,
    pub effect_type: u8,
    pub jihuoqixue: String,
    pub base_damage1: u32,
    pub base_damage2: u32,
    pub atk_xishu: f32,
    pub watk_xishu: u32,
    pub hit_up: u32,
    pub huixin_up: u32,
    pub huixiao_up: u32,
    pub wushifangyu: u32,
    pub wushihuajin: u32,
    pub wushijianshang: u32,
    pub zhenshishanghai: u32,
    pub lost_hp_zhenshishanghai: f32,
    pub dot_flag: u8,
    pub dot_interval: f32,
    pub dot_duration: f32,
    pub dot_up: f32,
    // ---- #92 时序字段（旧数据缺省 = 瞬发 / 占普通 GCD / 无调息 / 无充能） ----
    /// 吟唱/读条时长（秒，0=瞬发）
    #[serde(default)]
    pub cast_time: f32,
    /// GCD 类型（0=占普通 GCD，1=不占 GCD，2=固定 GCD 不受加速）
    #[serde(default)]
    pub gcd_kind: u8,
    /// 调息时间（秒，0=无调息）
    #[serde(default)]
    pub cooldown: f32,
    /// 充能层数（0 = 普通冷却；>=1 = 最大充能次数，使用 recharge 恢复）
    #[serde(default)]
    pub charges: u8,
    /// 充能恢复间隔（秒）
    #[serde(default)]
    pub recharge: f32,
    /// 调息是否受加速影响
    #[serde(default)]
    pub cd_haste: bool,
    /// 引导技能（吟唱期间持续结算，伤害按引导跳数）
    #[serde(default)]
    pub channel: bool,
    /// 仅提供 buff 的技能所施加的 buff id（空 = 不施加）
    #[serde(default)]
    pub buff_id: String,
    /// 该技能所在形态的基础公共 CD（秒；0 = 用门派默认）
    #[serde(default)]
    pub gcd_base: f32,
    /// GCD 分组（0 = 默认组；双形态门派另一套技能用 1，两组 GCD 互相独立）
    #[serde(default)]
    pub gcd_group: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionInfoDTO {
    pub level: u32,
    pub season: u32,
    pub modified: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillEditorDataDTO {
    pub xinfa: XinfaConfigDTO,
    pub version: Option<VersionInfoDTO>,
    /// None/旧请求不删除已有规则；规则由各心法 TOML 维护。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<AttributeRules>,
    pub skills: Vec<SkillEditorItemDTO>,
}

// ============ FFI 宿主事件回调表（动态模式） ============

/// 宿主事件回调函数表（jpcg_set_host_events 传入）
/// 布局与 jpcg_core::ffi::HostEventsTable 完全一致，作为跨端 C ABI 契约单源。
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HostEventsTable {
    /// 进度上报：on_progress(event_json: *const c_char)
    pub on_progress: Option<unsafe extern "C" fn(event_json: *const std::os::raw::c_char)>,
    /// 请求宿主退出：request_exit() -> c_int（0 成功，非 0 失败）
    pub request_exit: Option<unsafe extern "C" fn() -> std::os::raw::c_int>,
    /// 注入 updater 二进制绝对路径：updater_path() -> *const c_char（null 表示未提供）
    pub updater_path: Option<unsafe extern "C" fn() -> *const std::os::raw::c_char>,
}

use serde::{Deserialize, Serialize};

// ============ #92 时序引擎 / 战斗数据 DTO ============
// 与 `data/combat/{门派}.toml`、`jpcg_combo::timeline` 一一对应；
// 前端（TS）与后端（Rust）共用这一份契约，避免两套类型。

/// 技能唯一键（skill_id, sub_id）；JSON 中表现为 [id, sub] 数组
pub type SkillKeyDTO = (u32, u32);

/// 形态定义（`[[form]]`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FormDefDTO {
    pub id: String,
    pub name: String,
    /// 该形态基础 GCD（秒；0 = 门派默认）
    pub gcd_base: f32,
    pub initial: bool,
    /// 该形态可用技能（空 = 不限）
    pub skills: Vec<SkillKeyDTO>,
}

/// 技能施放副作用
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OnCastDTO {
    /// 施放后切换到的器形 id（如 "jian" / "qin"）
    pub switch_instrument: Option<String>,
    /// 施放后切换到的曲风 id（0 秒状态事件）
    pub switch_qifeng: Option<String>,
    pub gcd_lanes: Vec<String>,
    pub lock_skills: Vec<SkillKeyDTO>,
    /// 施放时让这些技能的 DOT 下一跳立刻生效（羽：商/角）
    pub advance_dot_targets: Vec<SkillKeyDTO>,
    /// 提前跳数
    pub advance_dot_ticks: u8,
}

/// 单技能时序 overlay（`[[skill_combat]]`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillCombatDTO {
    pub skill_id: u32,
    pub sub_id: u32,
    /// 要求当前器形（空 = 任意）
    pub requires_instrument: Option<String>,
    /// 要求当前曲风（空 = 任意；如 变宫/变徵 需要 gaoshan）
    pub requires_qifeng: Option<String>,
    pub gcd_lane: String,
    pub on_cast: OnCastDTO,
    // ---- 可选覆盖（None = 沿用技能数据） ----
    pub cast_time: Option<f32>,
    pub gcd_base: Option<f32>,
    pub gcd_kind: Option<u8>,
    pub cooldown: Option<f32>,
    pub charges: Option<u8>,
    pub recharge: Option<f32>,
    pub cd_haste: Option<bool>,
    pub channel: Option<bool>,
    /// 引导/倒读条段数（None = 1 段）
    pub channel_ticks: Option<u8>,
    pub buff_id: Option<String>,
    /// 激活所需奇穴 id（空 = 不限）
    pub jihuoqixue: Option<String>,
    /// 门控奇穴解析出的 qixue id（后端按 qixue 表解析，前端直接用 id 判断）
    #[serde(default)]
    pub jihuoqixue_id: Option<String>,
    /// 施放获得曲风点数（空 = 0）
    pub qifeng_gain: Option<u8>,
    /// ---- DOT（独立于本体伤害） ----
    pub dot_interval: Option<f32>,
    pub dot_duration: Option<f32>,
    pub dot_tick_base1: Option<u32>,
    pub dot_tick_base2: Option<u32>,
    pub dot_tick_xishu: Option<f32>,
    /// 施放消耗全部曲风（空 = false）
    pub spend_all_qifeng: Option<bool>,
    /// 已配对的秘籍 id（后端组装，前端只按 id 查 MijiDefDTO）
    #[serde(default)]
    pub miji_ids: Vec<String>,
}

/// 定时 buff 效果条目（百分比）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffEffectDTO {
    BaseAtkPct(f32),
    HuixinPct(f32),
    HuixiaoPct(f32),
    PofangPct(f32),
    WushiFangyuPct(f32),
    ShanghaiPct(f32),
    /// 加速率加成（百分比，如 2.0 = +2% 加速率）
    JiasuPct(f32),
}

/// 带持续时间的 buff 定义（`[[buff]]`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimedBuffDefDTO {
    pub id: String,
    pub name: String,
    /// 持续时间（秒；0 = 常驻）
    pub duration: f32,
    pub max_stacks: u8,
    pub refresh: bool,
    /// 该 buff 的加速能否突破 25% 上限
    pub ignore_cap: bool,
    pub effects: Vec<BuffEffectDTO>,
}

/// 技能修正项（秘籍 / 奇穴共用）
/// 奇穴运行时钩子（第 3 层）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QixueHookDTO {
    FlatDamageOnCast {
        damage: f32,
        #[serde(default)]
        per_qifeng_spent: bool,
    },
    DamagePctPerQifeng {
        pct: f32,
    },
    /// 该技能每次 DOT 跳，使 targets 指定技能的下一跳立刻生效
    AccelerateDot {
        targets: Vec<SkillKeyDTO>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillModifierDTO {
    BaseDamageAdd(u32),
    BaseDamageMul(f32),
    AtkXishuAdd(f32),
    HitUpAdd(u32),
    HuixinUpAdd(u32),
    HuixiaoUpAdd(u32),
    WushifangyuAdd(u32),
    CastTimeMul(f32),
    CastTimeAdd(f32),
    CooldownMul(f32),
    DotDurationMul(f32),
    DotIntervalMul(f32),
    CooldownAdd(f32),
    DotTicksAdd(i32),
    /// Dot 每跳递增比例 +v
    #[serde(rename = "dot_up_add")]
    DotUpAdd(f32),
}

/// 秘籍定义（`[[miji]]`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MijiDefDTO {
    pub id: String,
    pub name: String,
    pub skill_id: u32,
    pub sub_id: u32,
    /// 共享作用技能列表 `[[skill_id, sub_id], ...]`（非空时优先于 skill_id/sub_id）。
    #[serde(default)]
    pub applies_to: Vec<(u32, u32)>,
    pub modifiers: Vec<SkillModifierDTO>,
    /// 后端生成的效果文案（前端只渲染，不翻译 modifiers）
    #[serde(default)]
    pub effects: Vec<String>,
}

/// 奇穴定义（`[[qixue]]`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct QixueDefDTO {
    pub id: String,
    pub name: String,
    pub slot: u8,
    pub replaces_skill_id: u32,
    pub replaces_sub_id: u32,
    /// 作用技能列表（(skill_id, sub_id)；空 = 用 replaces_skill_id/sub_id）
    pub applies_to: Vec<(u32, u32)>,
    /// 该奇穴解锁的技能（(skill_id, sub_id)）
    pub grants_skills: Vec<(u32, u32)>,
    /// 运行时钩子（第 3 层）
    #[serde(default)]
    pub hooks: Vec<QixueHookDTO>,
    /// 暂不建模（PvE/待补数据）
    #[serde(default)]
    pub shelved: bool,
    /// 替换后的独立技能条目（None = 只施加 modifiers）
    pub replacement: Option<SkillPoolItemDTO>,
    pub modifiers: Vec<SkillModifierDTO>,
    /// 显示分组标签（第一重~第六重 / 混选），后端算好
    #[serde(default)]
    pub group_label: String,
    /// 同组一起开关的多个 id（如冻泉 + 冻泉·宫）
    #[serde(default)]
    pub group_ids: Vec<String>,
}

/// 战斗数据元信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatMetaDTO {
    pub profession: String,
    pub version: String,
    pub note: String,
    /// 面板规则：每个技能最多装备的秘籍数
    #[serde(default = "default_miji_cap")]
    pub miji_per_skill_cap: u8,
    /// 混选奇穴最多选几格
    #[serde(default = "default_mixed_cap")]
    pub mixed_qixue_cap: u8,
    /// 从第几号槽位开始是混选池
    #[serde(default = "default_mixed_slot_from")]
    pub mixed_slot_from: u8,
}

pub fn default_miji_cap() -> u8 {
    4
}

pub fn default_mixed_cap() -> u8 {
    3
}

pub fn default_mixed_slot_from() -> u8 {
    7
}

/// `data/combat/{门派}.toml` 的完整 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatDataDTO {
    pub meta: CombatMetaDTO,
    pub form: Vec<FormDefDTO>,
    /// 曲风 id 列表（yangchun / gaoshan / pingsha）
    pub qifeng: Vec<String>,
    pub skill_combat: Vec<SkillCombatDTO>,
    pub buff: Vec<TimedBuffDefDTO>,
    pub miji: Vec<MijiDefDTO>,
    pub qixue: Vec<QixueDefDTO>,
}

/// 时间轴动作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TimelineActionDTO {
    Cast {
        skill_id: u32,
        sub_id: u32,
        /// 技能名：用于在 (skill_id, sub_id) 对应多个变体（如「入月·N 层」）时消歧
        #[serde(default)]
        skill_name: Option<String>,
        #[serde(default)]
        overrides: Option<StepOverrideDTO>,
    },
    SwitchForm {
        form: String,
    },
    /// 计算开始/结束标记（只影响击杀率测量窗口；"start" | "end"）
    Flag {
        flag: String,
    },
}

/// 时间轴计算请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRequestDTO {
    pub skills: Vec<SkillPoolItemDTO>,
    #[serde(default)]
    pub forms: Vec<FormDefDTO>,
    #[serde(default)]
    pub timed_buffs: Vec<TimedBuffDefDTO>,
    #[serde(default)]
    pub actions: Vec<TimelineActionDTO>,
    /// 模拟窗口（秒；0 = 默认 300 秒）
    #[serde(default)]
    pub window: f32,
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa: XinfaConfigDTO,
    pub buff: BuffConfigDTO,
    pub coeff: CoefficientConfigDTO,
    /// 数值集 id（None = 默认值集）
    #[serde(default)]
    pub value_set: Option<String>,
    /// 激活的奇穴 id 列表（可选；激活后按 [[qixue]] 的 modifiers 生效）
    #[serde(default)]
    pub qixue: Vec<String>,
    /// 激活的秘籍 id 列表（可选；激活后按 [[miji]] 的 modifiers 生效）
    #[serde(default)]
    pub miji: Vec<String>,
    /// 初始器形 id（空 = 取 combat.forms 里 initial 的）
    #[serde(default)]
    pub initial_instrument: Option<String>,
    /// 初始曲风 id（空 = 不限）
    #[serde(default)]
    pub initial_qifeng: String,
    /// 战斗时序 overlay（可选；提供时按 overlay 套形态 / GCD 通道 / on_cast）
    #[serde(default)]
    pub combat: Option<CombatDataDTO>,
    /// 击杀率蒙特卡洛样本数（0 = 不跑 MC，纯确定性）
    #[serde(default)]
    pub samples: u32,
    /// 击杀率随机种子（None = 每次随机）
    #[serde(default)]
    pub seed: Option<u64>,
}

/// 时间轴事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TimelineEventKindDTO {
    Cast {
        skill_name: String,
        damage: u32,
        hit_time: f32,
        /// 起手时刻；外层 time 是实际结算时刻。
        #[serde(default)]
        start_time: f32,
        /// 本次施放占用的 GCD 秒数（不占 GCD 为 0）
        #[serde(default)]
        gcd_secs: f32,
        /// 该 GCD 通道就绪时刻
        #[serde(default)]
        gcd_ready_at: f32,
    },
    /// 引导/倒读条单段
    ChannelTick {
        skill_name: String,
        index: u32,
        damage: u32,
    },
    DotTick {
        skill_name: String,
        index: u32,
        damage: u32,
        /// 是否被洞天/羽提前生效
        #[serde(default)]
        advanced: bool,
    },
    FormSwitch {
        from: String,
        to: String,
    },
    /// 曲风切换（0 秒状态事件）
    QifengSwitch {
        from: String,
        to: String,
    },
    BuffGain {
        buff_id: String,
        stacks: u8,
    },
}

/// 窗口内时间轴日志，按实际结算时刻排序；Cast 起手单独保存在 start_time。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventDTO {
    pub time: f32,
    pub kind: TimelineEventKindDTO,
    /// 该事件发生后的事件累计伤害（后端算好，前端只显示）
    #[serde(default)]
    pub cumulative: f64,
}

/// 时间轴计算结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineResultDTO {
    pub events: Vec<TimelineEventDTO>,
    /// 每个动作的结算结果（与请求 actions 一一对应；前端直接渲染，不再推导）
    pub steps: Vec<TimelineStepDTO>,
    pub total_damage: f64,
    pub window_dps: f64,
    pub skipped_actions: u32,
    /// 序列执行完后的器形 / 曲风（后端算好，前端直接显示）
    pub final_instrument: String,
    pub final_qifeng: String,
    /// 窗口内击杀概率 0..1（samples=0 或未提供血量时为 0）
    #[serde(default)]
    pub final_kill_prob: f64,
    /// 累计击杀概率曲线：(动作序号 1..=n, 概率 0..1)
    #[serde(default)]
    pub kill_prob_curve: Vec<(usize, f64)>,
    /// 有效测量窗口起止（缺省已折算：开始=0，结束=最后一个动作 / 最后一次施放的 DOT 尾）
    #[serde(default)]
    pub kill_start: Option<f32>,
    #[serde(default)]
    pub kill_end: Option<f32>,
}

/// 单个动作的执行结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineStepDTO {
    /// 实际施放/切换时刻；被跳过为 None
    pub time: Option<f32>,
    pub gcd_secs: f32,
    pub gcd_ready_at: f32,
    pub legal: bool,
    pub instrument: String,
    pub qifeng: String,
    /// 本动作对应的技能名（计算开始/结束、器形切换为说明文字）
    #[serde(default)]
    pub skill_name: Option<String>,
    pub skipped_reason: String,
    #[serde(default)]
    pub required_instrument: Option<String>,
    #[serde(default)]
    pub required_qifeng: Option<String>,
    pub switches: Vec<TimelineStepSwitchDTO>,
}

/// 动作执行前的器形/曲风切换
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineStepSwitchDTO {
    /// "form" / "qifeng"
    pub switch_kind: String,
    pub from: String,
    pub to: String,
    pub time: f32,
}

/// 时间轴预览数据包：技能池（已套 combat overlay）+ combat 数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CombatBundleDTO {
    pub skills: Vec<SkillPoolItemDTO>,
    pub combat: CombatDataDTO,
}
