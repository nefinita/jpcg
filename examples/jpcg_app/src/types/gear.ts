/** 与 jpcg_gear::model（#94）保持同名形状；不另定义配装规则或计算公式。 */
import type { PlayerConfigDTO, CalculateRequest, ComboStepDTO, TimelineRequestDTO } from "./index";
export const GEAR_SLOTS = ["tou", "yi", "yao", "shou", "ku", "xie", "wuqi", "anqi", "xianglian", "yaozhui", "jie1", "jie2"] as const;
export type GearSlot = typeof GEAR_SLOTS[number];
export type GearSource = "mingjian" | "weiwang" | "baowu";
export type GearTier = "base" | "upgraded";
export interface GearStats { zhushuxing: number; gongji: number; qixue: number; huixin: number; huixiao: number; pofang: number; jiasu: number; yujin: number; huajin: number; wuqi_shanghai: number; tizhi?: number; liliang?: number; shenfa?: number; gengu?: number; yuanqi?: number; qixue_additional?: number }
export interface GearChoice { slot: GearSlot; source: GearSource; tier: GearTier; variant: number; school?: string | null }
export interface GearLoadout { school: string; revision: string; value_set: string; base: GearStats; choices: GearChoice[]; attribute_rules?: string | null; enchants?: Record<string, string>; wucai?: string }
export interface GearSchool { id: string; zhushuxing: "liliang" | "shenfa" | "gengu" | "yuanqi"; gongji_type: "neigong" | "waigong"; pofang_type: "neigong" | "waigong" }
export interface GearEnchantDef { slot: GearSlot; id: string; name: string; stats: GearStats }
export interface GearWucaiDef { id: string; name: string; stats: GearStats }
export interface GearTemplate { school: string; cross_school?: boolean; sockets?: GearStats[]; choice: GearChoice; green: "pofang" | "huixin" | "jiasu" | "yujin"; stats: GearStats; set: string | null; zhuangfen: number | null }
export interface GearCatalog {
  version: { schema: number; season: number; revision: string; value_set: string; sample: boolean; provenance: string };
  schools: GearSchool[];
  templates: GearTemplate[];
  sets: { id: string; source: GearSource; kind: "armor" | "accessory"; tiers: { tier: GearTier; bonus2: GearStats; bonus3: GearStats | null; cooldown4: { skill_id: number; seconds: number }[] }[] }[];
  enchants?: GearEnchantDef[];
  wucai?: GearWucaiDef[];
}
export interface GearAttributePanel { rule_id: string; source: string; verified_in_game: boolean; level?: number; value_set?: string; damage_kind?: "physical" | "solar" | "lunar" | "neutral" | "poison"; tizhi: number; liliang: number; shenfa: number; gengu: number; yuanqi: number; gongji_base: number; gongji_extra: number; gongji: number; huixin: number; pofang: number; qixue: number; qixue_base?: number; qixue_additional?: number }
export interface GearComposed { loadout: GearLoadout; profile: GearSchool; stats: GearStats; active_bonuses: { set: string; pieces: number; threshold: number; tier: GearTier }[]; cooldowns: { skill_id: number; seconds: number }[]; warnings: string[]; panel?: GearAttributePanel | null }
export interface GearComposition { composed: GearComposed; player: PlayerConfigDTO }
export const emptyGearStats = (): Required<GearStats> => ({ zhushuxing: 0, gongji: 0, qixue: 0, huixin: 0, huixiao: 0, pofang: 0, jiasu: 0, yujin: 0, huajin: 0, wuqi_shanghai: 0, tizhi: 0, liliang: 0, shenfa: 0, gengu: 0, yuanqi: 0, qixue_additional: 0 });

// ---- #90 配装优化契约（与 jpcg_gear::model 同名形状） ----
export interface GearBounds { min?: number; max?: number }
export interface GearConstraints {
  sources: GearSource[];
  allow_upgraded: boolean;
  locked: GearChoice[];
  banned: GearChoice[];
  required_sources: Record<string, GearSource>;
  banned_sources: Record<string, GearSource[]>;
  stats: Record<string, GearBounds>;
  allow_cross_source_2plus2: boolean;
  required_sets?: Record<string, number>;
}
export type GearTimelineMetric = "total_damage" | "window_dps" | "kill_probability";
/** 自动调优的选项（目标指标 + 约束）。 */
export interface OptimizeOptions {
  metric: "kill_probability" | "window_dps";
  arena4: boolean;
  hasteMin: number | null;
}
export type GearObjective =
  | { kind: "expected"; weights: Record<string, number> }
  | { kind: "kill_probability"; steps: ComboStepDTO[]; samples: number; seed: number; allow_unapplied_cd?: boolean }
  | { kind: "timeline"; request: TimelineRequestDTO; metric: GearTimelineMetric };
export interface GearOptimizeRequest {
  loadout: GearLoadout;
  config: CalculateRequest;
  objective: GearObjective;
  constraints: GearConstraints;
  max_combinations: number;
}
export interface GearCandidate { composed: GearComposed; score: number }
export interface GearProgress { visited: number; evaluated: number; combinations: number }
export interface GearOptimizeResult {
  best: GearCandidate | null;
  baseline: GearCandidate;
  delta: number | null;
  pareto: GearCandidate[];
  progress: GearProgress;
  complete: boolean;
  stop_reason: string;
  warnings: string[];
}
