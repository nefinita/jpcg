import { describe, expect, it } from "vitest";
import { checkFormLegality } from "./formLegality";
import type { CombatDataDTO, ComboStepDTO, SkillPoolItemDTO } from "../types";

const skill = (id: number, name: string): SkillPoolItemDTO => ({ skill_id: id, sub_id: 0, skill_name: name } as SkillPoolItemDTO);
const step = (s: SkillPoolItemDTO): ComboStepDTO => ({ skill: s } as ComboStepDTO);

const combat = {
  meta: { profession: "mowen", version: "v1", note: "" },
  form: [
    { id: "qu-yangchun", name: "阳春白雪", gcd_base: 1.5, initial: true, skills: [] },
    { id: "jian", name: "持剑", gcd_base: 1.5, initial: false, skills: [] },
  ],
  skill_combat: [
    { skill_id: 1, sub_id: 0, form: "qu-yangchun", gcd_lane: "", on_cast: { switch_form: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 2, sub_id: 0, form: "jian", gcd_lane: "", on_cast: { switch_form: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 3, sub_id: 0, form: null, gcd_lane: "", on_cast: { switch_form: "jian", gcd_lanes: [], lock_skills: [] } },
  ],
  buff: [], miji: [], qixue: [],
} as unknown as CombatDataDTO;

describe("checkFormLegality", () => {
  it("形态不符的技能标记为非法并给出所需形态", () => {
    const steps = [step(skill(1, "宫")), step(skill(2, "剑·宫"))];
    const out = checkFormLegality(steps, combat);
    expect(out[0]).toMatchObject({ legal: true, form: "阳春白雪" });
    expect(out[1]).toMatchObject({ legal: false, requiredForm: "持剑" });
  });

  it("按 on_cast 切换形态后后续技能合法", () => {
    const steps = [step(skill(3, "清绝影歌")), step(skill(2, "剑·宫"))];
    const out = checkFormLegality(steps, combat);
    expect(out.map((s) => s.legal)).toEqual([true, true]);
    expect(out[1].form).toBe("持剑");
  });

  it("缺少 overlay 时不拦截", () => {
    expect(checkFormLegality([step(skill(9, "任意"))], null)).toEqual([{ legal: true, form: "" }]);
  });
});
