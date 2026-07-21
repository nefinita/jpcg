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
}

export interface SkillPoolItemDTO {
  skill_name: string;
  skill_id: number;
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

export interface AttributeConfigDocumentDTO {
  profession: string;
  file_name: string;
  content: string;
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

export interface FormData {
  xinfa: string;
  player: Record<string, number>;
  hostile: Record<string, number>;
  xinfa_config: {
    profession: string;
    xinfa_name: string;
    xinfa_nom: string;
    atk_up: number;
    pofang_up: number;
    huixin_up: number;
  };
  buff: BuffConfigDTO;
  coefficient: CoefficientConfigDTO;
}
