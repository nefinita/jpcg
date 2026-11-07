import type { FormData, SequenceItem, CombatBundleDTO, TimelineRequestDTO } from "../types";
import { toCalculateRequest } from "./normalize";

/** 由当前面板 + 战斗 overlay + 连招序列组装时间轴请求（排轴器与配装调优共用）。 */
export function buildTimelineRequest(
  form: FormData,
  bundle: CombatBundleDTO,
  sequence: SequenceItem[],
  miji: string[],
  qixue: string[],
  samples: number,
  seed?: number | null,
): TimelineRequestDTO {
  const req = toCalculateRequest(form);
  return {
    skills: bundle.skills,
    forms: bundle.combat.form,
    timed_buffs: bundle.combat.buff,
    actions: sequence.map((s) => (s.kind === "flag"
      ? { kind: "flag" as const, flag: s.flag }
      : { kind: "cast" as const, skill_id: s.skill.skill_id, sub_id: s.skill.sub_id ?? 0, skill_name: s.skill.skill_name })),
    window: 300,
    player: req.player,
    hostile: req.hostile,
    xinfa: req.xinfa_config,
    buff: req.buff,
    coeff: req.coefficient,
    value_set: req.value_set ?? null,
    combat: bundle.combat,
    initial_instrument: bundle.combat.form.find((f) => f.initial)?.id ?? bundle.combat.form[0]?.id ?? null,
    initial_qifeng: bundle.combat.qifeng?.[0] ?? "",
    miji,
    qixue,
    samples,
    seed: seed ?? null,
  };
}
