import type { XinfaEntry } from "../types";

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
];

export const STORAGE_KEYS = {
  config: "jpcg_user_config",
  theme: "jpcg_theme",
  lastXinfa: "jpcg_last_xinfa",
};

export const FORUM_URL = "http://localhost:8080";
