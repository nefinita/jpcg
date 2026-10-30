export interface PlayerConfigDTO {
  jcsx: string;
  jichu_shuxing: number;
  jichu_gongji: number;
  huixin_dengji: number;
  huixin_xiaoguo: number;
  pofang_dengji: number;
  wuqi_shanghai: number;
  zuizhong_gongji?: number;
  /** 加速等级（#92 时序引擎；0=无加速） */
  jiasu_dengji?: number;
}

export interface HostileConfigDTO {
  waigong_fangyu: number;
  neigong_fangyu: number;
  yujin_dengji: number;
  huajin_dengji: number;
  jianshang_bili: number;
  target_hp: number;
  /** 目标最大血量（追加真伤斩杀线基座）；0 = 未提供，回退 target_hp 满血模型 */
  max_hp: number;
  /** 目标当前血量（连招初始血量）；0 或 > max_hp = 满血 */
  current_hp: number;
}

export interface BuffConfigDTO {
  base_atk_pct: number;
  huixin_pct: number;
  huixiao_pct: number;
  pofang_pct: number;
  wushi_fangyu_pct: number;
  shanghai_pct: number;
  mode_is_point: boolean;
}

export interface CoefficientConfigDTO {
  pofang_xishu: number;
  huixin_xishu: number;
  huixiao_xishu: number;
  /** 御劲 → 目标会心率减免分母 */
  yujin_xishu: number;
  /** 御劲 → 目标会心伤害减免分母 */
  yuhui_xishu: number;
  huajin_xishu: number;
  fangyu_xishu: number;
  pvp_global_jianshang: number;
  /** 加速等级系数（加速率 = 加速等级 / 系数 / 100；0 = 不启用） */
  jiasu_xishu?: number;
}

export interface XinfaConfigDTO {
  profession: string;
  xinfa_name: string;
  xinfa_nom: string;
  atk_up: number;
  pofang_up: number;
  huixin_up: number;
  /** 门派基础公共 CD（秒；0/缺失 = 1.5） */
  gcd_base?: number;
}

export interface CalculateRequest {
  player: PlayerConfigDTO;
  hostile: HostileConfigDTO;
  xinfa_config: XinfaConfigDTO;
  buff: BuffConfigDTO;
  coefficient: CoefficientConfigDTO;
  /** 数值集 id（null/缺省 = 默认值集；见 data/values/index.toml） */
  value_set?: string | null;
}

/** 数值集信息（对应 data/values/index.toml 条目） */
export interface ValueSetDTO {
  id: string;
  name: string;
  level: number;
  is_default: boolean;
  /** "data"（本地 data/values）或 "builtin"（内置兜底） */
  source: string;
  available: boolean;
  /** 快照可用时的换算常数（用于 seed「系数设置」） */
  coefficient: CoefficientConfigDTO | null;
}

export interface SkillResultDTO {
  skill_name: string;
  y: number;
  b: number;
  i: number;
  n: number;
  h: number;
  q: number;
  dot_jumps: number[];
  has_critical_strike: boolean;
  zhenshishanghai: number;
  lost_hp_zhenshishanghai: number;
}

export interface SkillPoolItemDTO {
  skill_name: string;
  skill_id: number;
  sub_id: number;
  base_damage1: number;
  base_damage2: number;
  atk_xishu: number;
  watk_xishu: number;
  hit_up: number;
  huixin_up: number;
  huixiao_up: number;
  wushifangyu: number;
  wushihuajin: number;
  dot_flag: number;
  has_critical_strike: boolean;
  lost_hp_zhenshishanghai: number;
  // #92 时序字段（可选；旧数据缺失 = 默认值）
  cast_time?: number;
  gcd_kind?: number;
  cooldown?: number;
  charges?: number;
  recharge?: number;
  cd_haste?: boolean;
  channel?: boolean;
  buff_id?: string | null;
  gcd_base?: number;
  gcd_group?: number;
  dot_interval?: number;
  dot_duration?: number;
}

export interface StepOverrideDTO {
  base_damage_override: number | null;
  atk_xishu_override: number | null;
  jianshang_bili_override: number | null;
  wushihuajin_override: number | null;
  extra_atk_pct: number | null;
  gain_override: number | null;
  extra_crit_pct: number | null;
  extra_crit_dmg_pct: number | null;
}

export interface ComboStepDTO {
  skill: SkillPoolItemDTO;
  overrides: StepOverrideDTO | null;
}

export interface ComboPresetDTO {
  name: string;
  steps: ComboStepDTO[];
}

export interface ComboStepResultDTO {
  skill_name: string;
  g_damage: number;
  h_damage: number;
  q_damage: number;
  crit_rate: number;
  cumulative_mean_wan: number;
  kill_prob: number;
  dot_jumps: number[];
  has_critical_strike: boolean;
  zhenshishanghai: number;
  lost_hp_zhenshi_damage: number;
}

export interface ComboResultDTO {
  steps: ComboStepResultDTO[];
  total_expected_damage_wan: number;
  final_kill_prob: number;
  kill_prob_curve: [number, number][];
}

export interface UpdateCheckResult {
  current_app_version: string | null;
  latest_app_version: string | null;
  has_app_update: boolean;
  current_data_version: string | null;
  latest_data_version: string | null;
  has_data_update: boolean;
  data_files_to_update: string[];
  has_modules_update: boolean;
  modules_version: string | null;
  modules_files_to_update: ModulesFileEntry[];
}

export interface ModulesFileEntry {
  name: string;
  hash: string;
  hash_type: string;
  size: number;
}

export interface ModuleVersions {
  core: string;
  update: string;
  const: string;
  combo: string;
  app: string;
}

export interface UpdateProgressEvent {
  stage: string;
  message: string;
  progress: number;
  file: string;
}

export interface XinfaSummaryDTO {
  value: string;
  label: string;
  nom: string;
  version_label: string | null;
}

export interface ForumFileInfo {
  name: string;
  size: number;
  modified: string;
}

export interface XinfaEntry {
  value: string;
  label: string;
  icon: string;
  default?: boolean;
}

export interface SkillEditorItemDTO {
  skill_name: string;
  skill_id: number;
  sub_id: number;
  group: number;
  weapon_request: number;
  design_effect: number;
  kind_type: number;
  cast_mode: number;
  guaranteed_hit: boolean;
  has_critical_strike: boolean;
  effect_type: number;
  jihuoqixue: string;
  base_damage1: number;
  base_damage2: number;
  atk_xishu: number;
  watk_xishu: number;
  hit_up: number;
  huixin_up: number;
  huixiao_up: number;
  wushifangyu: number;
  wushihuajin: number;
  wushijianshang: number;
  zhenshishanghai: number;
  lost_hp_zhenshishanghai: number;
  dot_flag: number;
  dot_interval: number;
  dot_duration: number;
  dot_up: number;
}

export interface VersionInfoDTO {
  level: number;
  season: number;
  modified: number;
}

export interface SkillEditorDataDTO {
  xinfa: {
    profession: string;
    xinfa_name: string;
    xinfa_nom: string;
    atk_up: number;
    pofang_up: number;
    huixin_up: number;
  };
  version: VersionInfoDTO | null;
  skills: SkillEditorItemDTO[];
}

export interface SkillDerivativeDTO {
  skill_name: string;
  derivative: number;
}

export interface DerivativeEntryDTO {
  attr_name: string;
  attr_id: string;
  current_value: number;
  total_derivative: number;
  per_skill: SkillDerivativeDTO[];
}

export interface CritVsPofangDTO {
  better: string;
  huixin_total: number;
  pofang_total: number;
}

export interface TopAttrDTO {
  attr_name: string;
  attr_id: string;
  total_derivative: number;
}

export interface OptimizeRecommendationDTO {
  crit_vs_pofang: CritVsPofangDTO;
  top3: TopAttrDTO[];
}

export interface DerivativesOutputDTO {
  derivatives: DerivativeEntryDTO[];
  recommendation: OptimizeRecommendationDTO;
}

export interface FormData {
  xinfa: string;
  player: Record<string, number | string>;
  hostile: Record<string, number | string>;
  xinfa_config: {
    profession: string;
    xinfa_name: string;
    xinfa_nom: string;
    atk_up: number;
    pofang_up: number;
    huixin_up: number;
  };
  buff: Record<string, number | string | boolean>;
  coefficient: Record<string, number | string>;
  /** 选中的数值集 id（null/缺省 = 默认值集） */
  value_set?: string | null;
}

// ============ #92 战斗时序（时间轴） ============

export type SkillKeyDTO = [number, number];

export interface FormDefDTO {
  id: string;
  name: string;
  gcd_base: number;
  initial: boolean;
  skills: SkillKeyDTO[];
}

export interface OnCastDTO {
  switch_form: string | null;
  gcd_lanes: string[];
  lock_skills: SkillKeyDTO[];
}

export interface SkillCombatDTO {
  skill_id: number;
  sub_id: number;
  form: string | null;
  gcd_lane: string;
  on_cast: OnCastDTO;
  cast_time: number | null;
  gcd_base: number | null;
  gcd_kind: number | null;
  cooldown: number | null;
  charges: number | null;
  recharge: number | null;
  cd_haste: boolean | null;
  channel: boolean | null;
  buff_id: string | null;
}

export type BuffEffectDTO =
  | { base_atk_pct: number }
  | { huixin_pct: number }
  | { huixiao_pct: number }
  | { pofang_pct: number }
  | { wushi_fangyu_pct: number }
  | { shanghai_pct: number }
  | { jiasu_pct: number };

export interface TimedBuffDefDTO {
  id: string;
  name: string;
  duration: number;
  max_stacks: number;
  refresh: boolean;
  ignore_cap: boolean;
  effects: BuffEffectDTO[];
}

export interface CombatMetaDTO {
  profession: string;
  version: string;
  note: string;
}

export interface CombatDataDTO {
  meta: CombatMetaDTO;
  form: FormDefDTO[];
  skill_combat: SkillCombatDTO[];
  buff: TimedBuffDefDTO[];
  miji: unknown[];
  qixue: unknown[];
}

export interface CombatBundleDTO {
  skills: SkillPoolItemDTO[];
  combat: CombatDataDTO;
}

export type TimelineActionDTO =
  | { kind: "cast"; skill_id: number; sub_id: number }
  | { kind: "switch_form"; form: string };

export interface TimelineRequestDTO {
  skills: SkillPoolItemDTO[];
  forms: FormDefDTO[];
  timed_buffs: TimedBuffDefDTO[];
  actions: TimelineActionDTO[];
  window: number;
  player: PlayerConfigDTO;
  hostile: HostileConfigDTO;
  xinfa: XinfaConfigDTO;
  buff: BuffConfigDTO;
  coeff: CoefficientConfigDTO;
  value_set?: string | null;
  combat?: CombatDataDTO | null;
}

export type TimelineEventKindDTO =
  | { cast: { skill_name: string; damage: number; hit_time: number } }
  | { dot_tick: { skill_name: string; index: number; damage: number } }
  | { form_switch: { from: string; to: string } }
  | { buff_gain: { buff_id: string; stacks: number } };

export interface TimelineEventDTO {
  time: number;
  kind: TimelineEventKindDTO;
}

export interface TimelineResultDTO {
  events: TimelineEventDTO[];
  total_damage: number;
  window_dps: number;
  skipped_actions: number;
}
