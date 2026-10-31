import { describe, expect, it } from "vitest";
import { fixture } from "../test/gear-fixture";
import { defaultCockpit, cockpitSchema } from "./cockpit";
import { emptyGearStats } from "../types/gear";
import { gearInputKey, gearPlayerForm, initialLoadout, isGearComposition, isGearLoadout, PROFESSION_ATTRIBUTE_RULES } from "./gear";
import { toCalculateRequest } from "../utils/normalize";

describe("配装属性换算契约", () => {
  it("旧存档缺少五维及规则字段时仍能完整恢复，且不自动开启新规则", () => {
    const legacy = fixture.cases[0].composed.loadout;
    expect(legacy.base.tizhi).toBeUndefined();
    expect(isGearLoadout(legacy)).toBe(true);
    expect(isGearComposition(fixture.cases[0])).toBe(true);
    const state = defaultCockpit(); state.loadouts.mowen = legacy;
    expect(cockpitSchema.validate(JSON.parse(JSON.stringify(state)))).toBe(true);
    expect(initialLoadout(fixture.catalog, "mowen")?.attribute_rules).toBeUndefined();
  });

  it("五维输入与规则参与持久化和计算缓存键，拒绝非法点数", () => {
    const legacy = fixture.cases[0].composed.loadout;
    const next = { ...legacy, attribute_rules: PROFESSION_ATTRIBUTE_RULES, base: { ...emptyGearStats(), ...legacy.base, tizhi: 123, gengu: 3, qixue_additional: 456 } };
    expect(isGearLoadout(JSON.parse(JSON.stringify(next)))).toBe(true);
    expect(gearInputKey(defaultCockpit().form, legacy)).not.toBe(gearInputKey(defaultCockpit().form, next));
    for (const tizhi of [-1, 0.5, 4294967296, NaN, "12"]) {
      expect(isGearLoadout({ ...next, base: { ...next.base, tizhi } })).toBe(false);
      expect(isGearLoadout({ ...next, base: { ...next.base, qixue_additional: tizhi } })).toBe(false);
    }
  });

  it("将换算攻击和加速传入伤害请求，同时保留旧模式的缺失语义", () => {
    const form = defaultCockpit().form;
    const raw = fixture.cases[0].player;
    for (const extra of [0, 2007]) {
      const player = gearPlayerForm({ ...raw, zhuanhua_gongji: extra, jiasu_dengji: 124 });
      const request = toCalculateRequest({ ...form, player });
      expect(request.player.zhuanhua_gongji).toBe(extra);
      expect(request.player.jiasu_dengji).toBe(124);
    }
    for (const extra of [undefined, null]) {
      const player = gearPlayerForm({ ...raw, zhuanhua_gongji: extra });
      expect(toCalculateRequest({ ...form, player }).player).not.toHaveProperty("zhuanhua_gongji");
    }
  });
});
