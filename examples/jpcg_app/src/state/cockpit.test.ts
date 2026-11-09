import { expect, it } from "vitest";
import { cockpitSchema, defaultCockpit } from "./cockpit";

it("旧 v2 快照不要求新增就绪字段，v1 迁移保留自定义表单", () => {
  const old = { ...defaultCockpit(), manualReady: true };
  old.form.coefficient.huixin_xishu = 777;
  expect(cockpitSchema.validate(old)).toBe(true);
  const migrated = cockpitSchema.migrate(old, 1);
  expect(cockpitSchema.validate(migrated)).toBe(true);
  expect(migrated).toMatchObject({ mode: "manual", manualReady: true, form: { coefficient: { huixin_xishu: 777 } } });
});

it("分别接受两项就绪状态并拒绝损坏标识", () => {
  const state = defaultCockpit();
  for (const xinfa of [false, true]) for (const coefficient of [false, true]) {
    expect(cockpitSchema.validate({ ...state, manualHydration: { xinfa, coefficient } })).toBe(true);
  }
  for (const manualHydration of [null, {}, { xinfa: "false", coefficient: true }, { xinfa: true, coefficient: 0 }]) {
    expect(cockpitSchema.validate({ ...state, manualHydration })).toBe(false);
  }
});

it("v2 迁移保留新时间轴结果，并迁移快照内旧技能序列", () => {
 const state = defaultCockpit();
 const result = {total_damage: 123, events: []};
 const old = {...state, combo: {inputKey: "current", result}, baseline: {form: state.form, results: [], comboResult: result,
 comboDraft: {sequence: [{skill: {skill_name: "旧技能", skill_id: 1, base_damage1: 0, base_damage2: 0, atk_xishu: 0, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false}, overrides: null}], presetName: "旧基准", selectedPreset: "", poolQuery: "", collapsedGroups: []}}};
 const migrated = cockpitSchema.migrate(old, 2);
 expect(cockpitSchema.validate(migrated)).toBe(true);
 expect(migrated).toMatchObject({combo: old.combo, baseline: {comboResult: result, comboDraft: {sequence: [{kind: "skill"}], presetName: "旧基准", miji: [], qixue: []}}});
});
