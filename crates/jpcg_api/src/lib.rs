// ============================================================================
// jpcg_api — JPCG 纯类型契约
// 提供跨进程共享的 DTO 类型（Tauri IPC / FFI JSON 双端共用）。
// 仅含序列化类型，不包含任何计算逻辑，不依赖 jpcg_core。
// 双端通过本 crate 保证 JSON 契约单一来源，防止漂移。
// ============================================================================

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
    pub huajin_xishu: f32,
    pub fangyu_xishu: f32,
    pub pvp_global_jianshang: f32,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa_config: XinfaConfigDTO,
    pub buff: BuffConfigDTO,
    pub coefficient: CoefficientConfigDTO,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComboPresetDTO {
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

// ============ 技能编辑器 DTO ============

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
