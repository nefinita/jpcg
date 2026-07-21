import { parse, stringify } from "smol-toml";
import {
  createSkill,
  type AttributeDraft,
  type SkillDraft,
} from "./model";

type UnknownTable = Record<string, unknown>;

function isTable(value: unknown): value is UnknownTable {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringOr(value: unknown, fallback = "") {
  return typeof value === "string" ? value : fallback;
}

function numberOr(value: unknown, fallback = 0) {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function booleanOr(value: unknown, fallback = false) {
  return typeof value === "boolean" ? value : fallback;
}

function normalizeSkill(value: unknown): SkillDraft {
  const raw = isTable(value) ? value : {};
  return createSkill({
    skill_name: stringOr(raw.skill_name, "未命名技能"),
    skill_id: numberOr(raw.skill_id),
    sub_id: numberOr(raw.sub_id),
    group: numberOr(raw.group),
    weapon_request: numberOr(raw.weapon_request),
    design_effect: numberOr(raw.design_effect),
    kind_type: numberOr(raw.kind_type),
    cast_mode: numberOr(raw.cast_mode),
    guaranteed_hit: booleanOr(raw.guaranteed_hit),
    has_critical_strike: booleanOr(raw.has_critical_strike),
    effect_type: numberOr(raw.effect_type),
    jihuoqixue: stringOr(raw.jihuoqixue),
    base_damage1: numberOr(raw.base_damage1),
    base_damage2: numberOr(raw.base_damage2),
    atk_xishu: numberOr(raw.atk_xishu),
    watk_xishu: numberOr(raw.watk_xishu),
    hit_up: numberOr(raw.hit_up),
    huixin_up: numberOr(raw.huixin_up),
    huixiao_up: numberOr(raw.huixiao_up),
    wushifangyu: numberOr(raw.wushifangyu),
    wushihuajin: numberOr(raw.wushihuajin),
    wushijianshang: numberOr(raw.wushijianshang),
    zhenshishanghai: numberOr(raw.zhenshishanghai),
    dot_flag: raw.dot_flag === true ? 1 : numberOr(raw.dot_flag),
    dot_up: numberOr(raw.dot_up),
  });
}

export function parseAttributeToml(source: string): AttributeDraft {
  const parsed = parse(source) as unknown;
  if (!isTable(parsed)) {
    throw new Error("TOML 顶层结构无效");
  }
  if (!Array.isArray(parsed.skill) || parsed.skill.length === 0) {
    throw new Error("没有找到 [[skill]] 技能条目");
  }

  const xinfa = isTable(parsed.xinfa) ? parsed.xinfa : parsed;
  const version = isTable(parsed.version)
    ? {
        level: numberOr(parsed.version.level),
        season: numberOr(parsed.version.season),
        modified: numberOr(parsed.version.modified),
      }
    : undefined;

  return {
    xinfa: {
      xinfa_name: stringOr(xinfa.xinfa_name, "莫问"),
      xinfa_nom: stringOr(xinfa.xinfa_nom, "根骨"),
      atk_up: numberOr(xinfa.atk_up),
      pofang_up: numberOr(xinfa.pofang_up),
      huixin_up: numberOr(xinfa.huixin_up),
    },
    version,
    skills: parsed.skill.map(normalizeSkill),
  };
}

export function serializeAttributeToml(draft: AttributeDraft) {
  const skills = draft.skills.map((skill) => {
    const serialized: UnknownTable = {
      skill_name: skill.skill_name,
      skill_id: skill.skill_id,
      sub_id: skill.sub_id,
      group: skill.group,
      weapon_request: skill.weapon_request,
      design_effect: skill.design_effect,
      kind_type: skill.kind_type,
      cast_mode: skill.cast_mode,
      guaranteed_hit: skill.guaranteed_hit,
      has_critical_strike: skill.has_critical_strike,
      effect_type: skill.effect_type,
      jihuoqixue: skill.jihuoqixue,
      base_damage1: skill.base_damage1,
      base_damage2: skill.base_damage2,
      atk_xishu: skill.atk_xishu,
      watk_xishu: skill.watk_xishu,
      hit_up: skill.hit_up,
      huixin_up: skill.huixin_up,
      huixiao_up: skill.huixiao_up,
      wushifangyu: skill.wushifangyu,
      wushihuajin: skill.wushihuajin,
      wushijianshang: skill.wushijianshang,
      zhenshishanghai: skill.zhenshishanghai,
      dot_flag: skill.dot_flag,
    };
    if (skill.dot_flag === 1) serialized.dot_up = skill.dot_up;
    return serialized;
  });

  return stringify({
    xinfa: {
      xinfa_name: draft.xinfa.xinfa_name,
      xinfa_nom: draft.xinfa.xinfa_nom,
      atk_up: draft.xinfa.atk_up,
      pofang_up: draft.xinfa.pofang_up,
      huixin_up: draft.xinfa.huixin_up,
    },
    ...(draft.version ? { version: draft.version } : {}),
    skill: skills,
  });
}
