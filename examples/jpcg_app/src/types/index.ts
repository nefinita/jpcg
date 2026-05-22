export interface PlayerConfigDTO {
  jcsx: string;
  jichu_shuxing: number;
  jichu_gongji: number;
  huixin_dengji: number;
  huixin_xiaoguo: number;
  pofang_dengji: number;
  wuqi_shanghai: number;
}

export interface HostileConfigDTO {
  waigong_fangyu: number;
  neigong_fangyu: number;
  yujin_dengji: number;
  huajin_dengji: number;
  jianshang_bili: number;
}

export interface XinfaConfigDTO {
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
    xinfa_name: string;
    xinfa_nom: string;
    atk_up: number;
    pofang_up: number;
    huixin_up: number;
  };
}
