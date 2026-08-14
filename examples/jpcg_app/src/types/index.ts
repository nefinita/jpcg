export interface PlayerConfigDTO {
  jcsx: string;
  jichu_shuxing: number;
  jichu_gongji: number;
  huixin_dengji: number;
  huixin_xiaoguo: number;
  pofang_dengji: number;
  wuqi_shanghai: number;
  zuizhong_gongji?: number;
}

export interface HostileConfigDTO {
  waigong_fangyu: number;
  neigong_fangyu: number;
  yujin_dengji: number;
  huajin_dengji: number;
  jianshang_bili: number;
  target_hp: number;
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
  huajin_xishu: number;
  fangyu_xishu: number;
  pvp_global_jianshang: number;
}

export interface XinfaConfigDTO {
  profession: string;
  xinfa_name: string;
  xinfa_nom: string;
  atk_up: number;
  pofang_up: number;
  huixin_up: number;
}

export interface CalculateRequest {
  player: PlayerConfigDTO;
  hostile: HostileConfigDTO;
  xinfa_config: XinfaConfigDTO;
  buff: BuffConfigDTO;
  coefficient: CoefficientConfigDTO;
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
}
