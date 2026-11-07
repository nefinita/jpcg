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
