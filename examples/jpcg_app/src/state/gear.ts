import { GEAR_SLOTS, emptyGearStats, type GearCatalog, type GearChoice, type GearLoadout, type GearComposition } from "../types/gear";
import type { FormData } from "../types";
import { toCalculateRequest } from "../utils/normalize";
export const slotNames = { tou: "头部", yi: "上衣", yao: "腰带", shou: "护手", ku: "裤子", xie: "鞋子", wuqi: "武器", anqi: "暗器", xianglian: "项链", yaozhui: "腰坠", jie1: "戒指一", jie2: "戒指二" };
export const sourceNames = { mingjian: "名剑币", weiwang: "威望", baowu: "宝物堆" };
export const greenNames = { pofang: "破防", huixin: "会心", jiasu: "加速", yujin: "御劲" };
export const statNames = { zhushuxing: "主属性", gongji: "攻击", qixue: "气血", huixin: "会心", huixiao: "会效", pofang: "破防", jiasu: "加速", yujin: "御劲", huajin: "化劲", wuqi_shanghai: "武器伤害" };
export const choiceKey = (c: GearChoice) => `${c.slot}:${c.source}:${c.tier}:${c.variant}`;
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
    && !!v.base && Object.keys(emptyGearStats()).every((k) => Number.isInteger(v.base![k as keyof typeof v.base]) && v.base![k as keyof typeof v.base] >= 0)
    && Array.isArray(v.choices) && v.choices.length === 12 && new Set(v.choices.map((c) => c?.slot)).size === 12
    && v.choices.every((c) => c && GEAR_SLOTS.includes(c.slot) && ["mingjian", "weiwang", "baowu"].includes(c.source) && ["base", "upgraded"].includes(c.tier) && [0, 1, 2].includes(c.variant));
}

export function isGearComposition(x: unknown): x is GearComposition {
  if (!x || typeof x !== "object") return false;
  const value = x as GearComposition;
  const numeric = (v: unknown) => typeof v === "number" && Number.isFinite(v);
  const composed = value.composed;
  return !!composed && isGearLoadout(composed.loadout) && !!composed.profile && typeof composed.profile.id === "string"
    && !!composed.stats && Object.keys(emptyGearStats()).every((k) => numeric(composed.stats[k as keyof typeof composed.stats]))
    && Array.isArray(composed.active_bonuses) && composed.active_bonuses.every((b) => b && typeof b.set === "string" && numeric(b.pieces) && numeric(b.threshold) && ["base", "upgraded"].includes(b.tier))
    && Array.isArray(composed.cooldowns) && composed.cooldowns.every((c) => c && numeric(c.skill_id) && numeric(c.seconds))
    && Array.isArray(composed.warnings) && composed.warnings.every((v) => typeof v === "string")
    && !!value.player && typeof value.player.jcsx === "string"
    && ["jichu_shuxing", "jichu_gongji", "huixin_dengji", "huixin_xiaoguo", "pofang_dengji", "wuqi_shanghai"].every((k) => numeric(value.player[k as keyof typeof value.player]));
}
