import type { FormData, SkillResultDTO, ComboStepDTO, ComboResultDTO } from "../types";
import { DEFAULT_BUFF, DEFAULT_COEFFICIENT, HOSTILE_FIELDS, PLAYER_FIELDS, STORAGE_KEYS, XINFA_LIST } from "../utils/constants";
import type { GearLoadout, GearComposition } from "../types/gear";
import { isGearLoadout, isGearComposition } from "./gear";
import { readPreference } from "../hooks/usePersistentState";

// 前端状态的版本封套；配装值使用 jpcg_gear 的既有契约。
export interface CalculationSnapshot {
  form: FormData; results: SkillResultDTO[];
  mode?: "manual" | "gear"; manualForm?: FormData;
  comboResult?: ComboResultDTO; comboDraft?: ComboDraft;
  loadout?: GearLoadout; composition?: GearComposition; inputKey?: string;
}
export interface CockpitState {
  mode: "manual" | "gear";
  loadouts: Record<string, GearLoadout>;
  form: FormData;
  manualReady: boolean;
  calculation: CalculationSnapshot | null;
  baseline: CalculationSnapshot | null;
  combo?: { inputKey: string; result: ComboResultDTO };
}
export interface ComboDraft {
  sequence: ComboStepDTO[];
  presetName: string;
  selectedPreset: string;
  poolQuery: string;
  collapsedGroups: string[];
}

export function defaultManualForm(): FormData {
  const stored = readPreference(STORAGE_KEYS.lastXinfa);
  const xinfa = XINFA_LIST.find((x) => x.value === stored) ?? XINFA_LIST.find((x) => x.default)!;
  return {
    xinfa: xinfa.value,
    player: Object.fromEntries(PLAYER_FIELDS.map((f) => [f.id, 0])),
    hostile: Object.fromEntries(HOSTILE_FIELDS.map((f) => [f.id, 0])),
    xinfa_config: { profession: xinfa.value, xinfa_name: xinfa.label, xinfa_nom: "gengu", atk_up: 0, pofang_up: 0, huixin_up: 0 },
    buff: { ...DEFAULT_BUFF }, coefficient: { ...DEFAULT_COEFFICIENT }, value_set: readPreference(STORAGE_KEYS.valueSet),
  };
}
export const defaultCockpit = (): CockpitState => ({ mode: "gear", loadouts: {}, form: defaultManualForm(), manualReady: false, calculation: null, baseline: null });
export const defaultComboDraft = (): ComboDraft => ({ sequence: [], presetName: "", selectedPreset: "", poolQuery: "", collapsedGroups: [] });

const record = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const number = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const numeric = (x: unknown) => number(x) || x === "";
const fields = (x: unknown, keys: string[], valid: (v: unknown) => boolean) => record(x) && keys.every((k) => valid(x[k]));
export function isManualForm(x: unknown): x is FormData {
  if (!record(x) || !record(x.xinfa_config)) return false;
  const config = x.xinfa_config;
  return typeof x.xinfa === "string"
    && ["profession", "xinfa_name", "xinfa_nom"].every((k) => typeof config[k] === "string")
    && fields(x.xinfa_config, ["atk_up", "pofang_up", "huixin_up"], number)
    && fields(x.player, PLAYER_FIELDS.map((f) => f.id), numeric)
    && fields(x.hostile, HOSTILE_FIELDS.map((f) => f.id), numeric)
    && fields(x.buff, Object.keys(DEFAULT_BUFF).filter((k) => k !== "mode_is_point"), numeric)
    && record(x.buff) && typeof x.buff.mode_is_point === "boolean"
    && fields(x.coefficient, Object.keys(DEFAULT_COEFFICIENT), numeric)
    && (x.value_set == null || typeof x.value_set === "string");
}
function isComboResult(x: unknown): x is ComboResultDTO {
  return record(x) && number(x.total_expected_damage_wan) && number(x.final_kill_prob) && Array.isArray(x.steps) && Array.isArray(x.kill_prob_curve);
}
function isSnapshot(x: unknown): x is CalculationSnapshot {
  return record(x) && isManualForm(x.form) && Array.isArray(x.results) && x.results.every((r: unknown) => record(r)
    && typeof r.skill_name === "string" && ["y", "b", "i", "n", "h", "q"].every((k) => number(r[k]))
    && Array.isArray(r.dot_jumps) && r.dot_jumps.every(number))
    && (x.mode === undefined || x.mode === "manual" || x.mode === "gear")
    && (x.manualForm === undefined || isManualForm(x.manualForm))
    && (x.loadout === undefined || isGearLoadout(x.loadout))
    && (x.composition === undefined || isGearComposition(x.composition))
    && (x.comboResult === undefined || isComboResult(x.comboResult))
    && (x.comboDraft === undefined || comboDraftSchema.validate(x.comboDraft))
    && (x.inputKey === undefined || typeof x.inputKey === "string");
}
export const cockpitSchema = {
  version: 2,
  validate: (x: unknown): x is CockpitState => record(x) && (x.mode === "manual" || x.mode === "gear") && record(x.loadouts) && Object.entries(x.loadouts).every(([school, loadout]) => isGearLoadout(loadout) && loadout.school === school) && typeof x.manualReady === "boolean"
    && (x.combo === undefined || (record(x.combo) && typeof x.combo.inputKey === "string" && isComboResult(x.combo.result)))
    && isManualForm(x.form) && (x.calculation === null || isSnapshot(x.calculation)) && (x.baseline === null || isSnapshot(x.baseline)),
  // #95/v1 仍按手动模式恢复，既有表单/结果/基准不丢。
  migrate: (x: unknown, version: number) => version <= 1 && record(x) ? { ...x, loadouts: {}, mode: "manual" } : null,
};
export const comboDraftSchema = {
  version: 1,
  validate: (x: unknown): x is ComboDraft => record(x) && Array.isArray(x.sequence) && x.sequence.every((s: unknown) => record(s)
    && record(s.skill) && typeof s.skill.skill_name === "string"
    && ["skill_id", "base_damage1", "base_damage2", "atk_xishu", "watk_xishu", "hit_up", "huixin_up", "huixiao_up", "wushifangyu", "wushihuajin", "dot_flag"].every((k) => record(s.skill) && number(s.skill[k]))
    && (s.skill.lost_hp_zhenshishanghai === undefined || number(s.skill.lost_hp_zhenshishanghai))
    && typeof s.skill.has_critical_strike === "boolean"
    && (s.overrides === null || (record(s.overrides) && Object.values(s.overrides).every((v) => v === null || number(v)))))
    && ["presetName", "selectedPreset", "poolQuery"].every((k) => typeof x[k] === "string")
    && Array.isArray(x.collapsedGroups) && x.collapsedGroups.every((s: unknown) => typeof s === "string"),
  migrate: (x: unknown, version: number) => version === 0 ? x : null,
};
