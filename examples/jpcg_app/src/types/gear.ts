/** 与 jpcg_gear::model（#94）保持同名形状；不另定义配装规则或计算公式。 */
import type { PlayerConfigDTO } from "./index";
export const GEAR_SLOTS = ["tou", "yi", "yao", "shou", "ku", "xie", "wuqi", "anqi", "xianglian", "yaozhui", "jie1", "jie2"] as const;
export type GearSlot = typeof GEAR_SLOTS[number];
export type GearSource = "mingjian" | "weiwang" | "baowu";
export type GearTier = "base" | "upgraded";
export interface GearStats { zhushuxing: number; gongji: number; qixue: number; huixin: number; huixiao: number; pofang: number; jiasu: number; yujin: number; huajin: number; wuqi_shanghai: number }
export interface GearChoice { slot: GearSlot; source: GearSource; tier: GearTier; variant: number }
export interface GearLoadout { school: string; revision: string; value_set: string; base: GearStats; choices: GearChoice[] }
export interface GearSchool { id: string; zhushuxing: "liliang" | "shenfa" | "gengu" | "yuanqi"; gongji_type: "neigong" | "waigong"; pofang_type: "neigong" | "waigong" }
export interface GearTemplate { school: string; choice: GearChoice; green: "pofang" | "huixin" | "jiasu" | "yujin"; stats: GearStats; set: string | null; zhuangfen: number | null }
export interface GearCatalog {
  version: { schema: number; season: number; revision: string; value_set: string; sample: boolean; provenance: string };
  schools: GearSchool[];
  templates: GearTemplate[];
  sets: { id: string; source: GearSource; kind: "armor" | "accessory"; tiers: { tier: GearTier; bonus2: GearStats; bonus3: GearStats | null; cooldown4: { skill_id: number; seconds: number }[] }[] }[];
}
export interface GearComposed { loadout: GearLoadout; profile: GearSchool; stats: GearStats; active_bonuses: { set: string; pieces: number; threshold: number; tier: GearTier }[]; cooldowns: { skill_id: number; seconds: number }[]; warnings: string[] }
export interface GearComposition { composed: GearComposed; player: PlayerConfigDTO }
export const emptyGearStats = (): GearStats => ({ zhushuxing: 0, gongji: 0, qixue: 0, huixin: 0, huixiao: 0, pofang: 0, jiasu: 0, yujin: 0, huajin: 0, wuqi_shanghai: 0 });
