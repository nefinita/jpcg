import { describe, expect, it } from "vitest";
import { checkFormLegality } from "./formLegality";
import type { CombatDataDTO, ComboStepDTO, SkillPoolItemDTO } from "../types";

const skill = (id: number, name: string): SkillPoolItemDTO => ({ skill_id: id, sub_id: 0, skill_name: name } as SkillPoolItemDTO);
const step = (s: SkillPoolItemDTO): ComboStepDTO => ({ skill: s } as ComboStepDTO);

const combat = {
  meta: { profession: "mowen", version: "v1", note: "" },
  form: [
    { id: "qin", name: "琴形态", gcd_base: 1.5, initial: true, skills: [] },
    { id: "jian", name: "剑形态", gcd_base: 1.5, initial: false, skills: [] },
  ],
  qifeng: ["yangchun", "gaoshan"],
  skill_combat: [
    { skill_id: 1, sub_id: 0, requires_instrument: "qin", requires_qifeng: null, gcd_lane: "", on_cast: { switch_instrument: null, switch_qifeng: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 2, sub_id: 0, requires_instrument: "jian", requires_qifeng: null, gcd_lane: "", on_cast: { switch_instrument: null, switch_qifeng: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 3, sub_id: 0, requires_instrument: null, requires_qifeng: null, gcd_lane: "", on_cast: { switch_instrument: "jian", switch_qifeng: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 4, sub_id: 0, requires_instrument: null, requires_qifeng: "gaoshan", gcd_lane: "", on_cast: { switch_instrument: null, switch_qifeng: null, gcd_lanes: [], lock_skills: [] } },
    { skill_id: 5, sub_id: 0, requires_instrument: null, requires_qifeng: null, gcd_lane: "", on_cast: { switch_instrument: null, switch_qifeng: "gaoshan", gcd_lanes: [], lock_skills: [] } },
  ],
  buff: [], miji: [], qixue: [],
} as unknown as CombatDataDTO;

describe("checkFormLegality", () => {
  it("器形不符的技能标记为非法并给出所需器形", () => {
    const steps = [step(skill(1, "宫")), step(skill(2, "剑·宫"))];
    const out = checkFormLegality(steps, combat);
    expect(out[0]).toMatchObject({ legal: true, form: "琴形态" });
    expect(out[1]).toMatchObject({ legal: false, requiredForm: "剑形态" });
  });

  it("曲风不符的技能标记为非法并给出所需曲风", () => {
    const out = checkFormLegality([step(skill(4, "变宫"))], combat);
    expect(out[0]).toMatchObject({ legal: false, requiredQifeng: "gaoshan" });
  });

  it("曲目技能切换曲风后变体可用", () => {
    const out = checkFormLegality([step(skill(5, "高山流水")), step(skill(4, "变宫"))], combat);
    expect(out.map((s) => s.legal)).toEqual([true, true]);
  });

  it("按 on_cast 切换器形后后续技能合法", () => {
    const steps = [step(skill(3, "清绝影歌")), step(skill(2, "剑·宫"))];
    const out = checkFormLegality(steps, combat);
    expect(out.map((s) => s.legal)).toEqual([true, true]);
    expect(out[1].form).toBe("剑形态");
  });

  it("缺少 overlay 时不拦截", () => {
    expect(checkFormLegality([step(skill(9, "任意"))], null)).toEqual([{ legal: true, form: "", qifeng: "" }]);
  });
});
