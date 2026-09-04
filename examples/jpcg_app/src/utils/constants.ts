import type { XinfaEntry, BuffConfigDTO, CoefficientConfigDTO } from "../types";

export const XINFA_LIST: XinfaEntry[] = [
  { value: "yinlong",  label: "隐龙诀",   icon: "" },
  { value: "wufang",   label: "无方",     icon: "" },
  { value: "cangjian", label: "藏剑山庄", icon: "" },
  { value: "qixiu",    label: "七秀坊",   icon: "" },
  { value: "wanhua",   label: "万花谷",   icon: "" },
  { value: "tiance",   label: "天策府",   icon: "" },
  { value: "shaolin",  label: "少林寺",   icon: "" },
  { value: "chunyang", label: "纯阳观",   icon: "" },
  { value: "wudu",     label: "五毒教",   icon: "" },
  { value: "tangmen",  label: "唐门",     icon: "" },
  { value: "mingjiao", label: "明教",     icon: "" },
  { value: "gaibang",  label: "丐帮",     icon: "" },
  { value: "cangyun",  label: "苍云",     icon: "" },
  { value: "mowen",    label: "莫问",     icon: "",   default: true },
  { value: "badao",    label: "霸刀山庄", icon: "" },
  { value: "penglai",  label: "蓬莱",     icon: "" },
  { value: "yantian",  label: "衍天宗",   icon: "" },
  { value: "yaozong",  label: "药宗",     icon: "" },
];

export const BUFF_FIELDS = [
  { id: "base_atk_pct",   label: "基础攻击%" },
  { id: "huixin_pct",     label: "会心提升%" },
  { id: "huixiao_pct",    label: "会效提升%" },
  { id: "pofang_pct",     label: "破防提升%" },
  { id: "wushi_fangyu_pct", label: "无视防御%" },
  { id: "shanghai_pct",   label: "伤害提升%" },
];

export const COEFFICIENT_FIELDS = [
  { id: "pofang_xishu",        label: "破防系数", default: 225957.6 },
  { id: "huixin_xishu",        label: "会心系数", default: 197703 },
  { id: "huixiao_xishu",       label: "会效系数", default: 72844.2 },
  { id: "yujin_xishu",         label: "御劲系数(会心)", default: 197703 },
  { id: "yuhui_xishu",         label: "御劲系数(会伤)", default: 55123.2 },
  { id: "huajin_xishu",        label: "化劲系数", default: 30115.8 },
  { id: "fangyu_xishu",        label: "防御系数", default: 126007.2 },
  { id: "pvp_global_jianshang", label: "PVP全局减伤", default: 0.9 },
];

export const DEFAULT_BUFF: BuffConfigDTO = {
  base_atk_pct: 0,
  huixin_pct: 0,
  huixiao_pct: 0,
  pofang_pct: 0,
  wushi_fangyu_pct: 0,
  shanghai_pct: 0,
  mode_is_point: false,
};

export const DEFAULT_COEFFICIENT: CoefficientConfigDTO = {
  pofang_xishu: 225957.6,
  huixin_xishu: 197703,
  huixiao_xishu: 72844.2,
  yujin_xishu: 197703,
  yuhui_xishu: 55123.2,
  huajin_xishu: 30115.8,
  fangyu_xishu: 126007.2,
  pvp_global_jianshang: 0.9,
};

export const PLAYER_FIELDS = [
  { id: "jichu_shuxing",   label: "基础属性",   min: 0, step: 1 },
  { id: "jichu_gongji",    label: "基础攻击",   min: 0, step: 1 },
  { id: "huixin_dengji",   label: "会心等级",   min: 0, step: 1 },
  { id: "huixin_xiaoguo",  label: "会心效果",   min: 0, step: 1 },
  { id: "pofang_dengji",   label: "破防等级",   min: 0, step: 1 },
  { id: "wuqi_shanghai",   label: "武器伤害",   min: 0, step: 1 },
];

export const HOSTILE_FIELDS = [
  { id: "waigong_fangyu",  label: "外功防御",     min: 0, step: 1 },
  { id: "neigong_fangyu",  label: "内功防御",     min: 0, step: 1 },
  { id: "yujin_dengji",    label: "御劲等级",     min: 0, step: 1 },
  { id: "huajin_dengji",   label: "化劲等级",     min: 0, step: 1 },
  { id: "jianshang_bili",  label: "减伤比例(%)",  min: 0, max: 100, step: 0.1 },
  { id: "target_hp",       label: "目标血量", min: 0, step: 1 },
  { id: "max_hp",          label: "目标最大血量", min: 0, step: 1 },
  { id: "current_hp",      label: "目标当前血量", min: 0, step: 1 },
];

export const STORAGE_KEYS = {
  config: "jpcg_user_config",
  theme: "jpcg_theme",
  lastXinfa: "jpcg_last_xinfa",
  betaChannel: "jpcg_beta_channel",
};

export const FORUM_URL = "https://forum.nefinita-ai.com";
export const GITHUB_ISSUES_URL = "https://github.com/nefinita/JPCG/issues/new";
