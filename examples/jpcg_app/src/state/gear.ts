import { GEAR_SLOTS, emptyGearStats, type GearCatalog, type GearChoice, type GearLoadout, type GearComposition, type GearStats, type GearAttributePanel } from "../types/gear";
import type { FormData, PlayerConfigDTO } from "../types";
import { toCalculateRequest } from "../utils/normalize";
export const slotNames = { tou: "头部", yi: "上衣", yao: "腰带", shou: "护手", ku: "裤子", xie: "鞋子", wuqi: "武器", anqi: "暗器", xianglian: "项链", yaozhui: "腰坠", jie1: "戒指一", jie2: "戒指二" };
export const sourceNames = { mingjian: "名剑币", weiwang: "威望", baowu: "宝物堆" };
export const greenNames = { pofang: "破防", huixin: "会心", jiasu: "加速", yujin: "御劲" };
export const statNames = { zhushuxing: "主属性", gongji: "攻击", qixue: "气血", huixin: "会心", huixiao: "会效", pofang: "破防", jiasu: "加速", yujin: "御劲", huajin: "化劲", wuqi_shanghai: "武器伤害", tizhi: "体质", liliang: "力道", shenfa: "身法", gengu: "根骨", yuanqi: "元气", qixue_additional: "额外气血上限" };
/** 由后端读取当前职业 TOML；前端不内置心法系数或推断规则是否已填写。 */
export const PROFESSION_ATTRIBUTE_RULES = "profession-toml";
const optionalStats = new Set(["tizhi", "liliang", "shenfa", "gengu", "yuanqi", "qixue_additional"]);
const points = (v: unknown): v is number => typeof v === "number" && Number.isInteger(v) && v >= 0 && v <= 0xffffffff;
function validStats(value: unknown): value is GearStats {
  if (!value || typeof value !== "object") return false;
  const stats = value as Record<string, unknown>;
  return Object.keys(emptyGearStats()).every((key) => (optionalStats.has(key) && stats[key] === undefined) || points(stats[key]));
}
function validPanel(value: unknown): value is GearAttributePanel {
  if (!value || typeof value !== "object") return false;
  const p = value as Record<string, unknown>;
  return typeof p.rule_id === "string" && typeof p.source === "string" && typeof p.verified_in_game === "boolean"
    && (p.damage_kind === undefined || ["physical", "solar", "lunar", "neutral", "poison"].includes(p.damage_kind as string))
    && (p.level === undefined || (points(p.level) && p.level > 0))
    && (p.value_set === undefined || typeof p.value_set === "string")
    && ["qixue_base", "qixue_additional"].every((key) => p[key] === undefined || points(p[key]))
    && ["tizhi", "liliang", "shenfa", "gengu", "yuanqi", "gongji_base", "gongji_extra", "gongji", "huixin", "pofang", "qixue"].every((key) => points(p[key]));
}
export const choiceKey = (c: GearChoice) => `${c.slot}:${c.source}:${c.tier}:${c.variant}`;
/** FormData 不存 null；缺失转化值不能归零，否则会关闭旧版主属性转攻击。 */
export function gearPlayerForm(player: PlayerConfigDTO): Record<string, number | string> {
  return Object.fromEntries(Object.entries(player).filter(([, value]) => value != null));
}
export function initialLoadout(catalog: GearCatalog, school: string): GearLoadout | null {
  const choices = GEAR_SLOTS.map((slot) => catalog.templates.find((t) => t.school === school && t.choice.slot === slot)?.choice);
  if (choices.some((c) => !c)) return null;
  return { school, revision: catalog.version.revision, value_set: catalog.version.value_set, base: emptyGearStats(), choices: choices as GearChoice[] };
}
export function gearInputKey(form: FormData, loadout: GearLoadout) {
  return JSON.stringify({ mode: "gear", loadout, context: { ...toCalculateRequest(form), player: null } });
}
export function isGearLoadout(x: unknown): x is GearLoadout {
  if (!x || typeof x !== "object") return false;
  const v = x as Partial<GearLoadout>;
  return typeof v.school === "string" && typeof v.revision === "string" && typeof v.value_set === "string"
    && validStats(v.base) && (v.attribute_rules == null || typeof v.attribute_rules === "string")
    && Array.isArray(v.choices) && v.choices.length === 12 && new Set(v.choices.map((c) => c?.slot)).size === 12
    && v.choices.every((c) => c && GEAR_SLOTS.includes(c.slot) && ["mingjian", "weiwang", "baowu"].includes(c.source) && ["base", "upgraded"].includes(c.tier) && [0, 1, 2].includes(c.variant));
}

export function isGearComposition(x: unknown): x is GearComposition {
  if (!x || typeof x !== "object") return false;
  const value = x as GearComposition;
  const numeric = (v: unknown) => typeof v === "number" && Number.isFinite(v);
  const composed = value.composed;
  return !!composed && isGearLoadout(composed.loadout) && !!composed.profile && typeof composed.profile.id === "string"
    && validStats(composed.stats) && (composed.panel == null || validPanel(composed.panel))
    && Array.isArray(composed.active_bonuses) && composed.active_bonuses.every((b) => b && typeof b.set === "string" && numeric(b.pieces) && numeric(b.threshold) && ["base", "upgraded"].includes(b.tier))
    && Array.isArray(composed.cooldowns) && composed.cooldowns.every((c) => c && numeric(c.skill_id) && numeric(c.seconds))
    && Array.isArray(composed.warnings) && composed.warnings.every((v) => typeof v === "string")
    && !!value.player && typeof value.player.jcsx === "string"
    && ["jichu_shuxing", "jichu_gongji", "huixin_dengji", "huixin_xiaoguo", "pofang_dengji", "wuqi_shanghai"].every((k) => numeric(value.player[k as keyof typeof value.player]))
    && (value.player.zhuanhua_gongji == null || points(value.player.zhuanhua_gongji));
}
