import type { CombatDataDTO, ComboStepDTO } from "../types";

export interface StepLegality {
  /** 该动作在当前器形/曲风下是否可用。 */
  legal: boolean;
  /** 执行到这一步之前所处的器形名。 */
  form: string;
  /** 执行到这一步之前所处的曲风 id。 */
  qifeng: string;
  /** 需要但当前不具备的器形名（非法时给出）。 */
  requiredForm?: string;
  /** 需要但当前不具备的曲风 id（非法时给出）。 */
  requiredQifeng?: string;
}

const formName = (combat: CombatDataDTO, id: string): string => combat.form.find((f) => f.id === id)?.name ?? id;

/** 按动作顺序模拟 器形 / 曲风 两个维度；不符的技能标记为非法。 */
export function checkFormLegality(sequence: ComboStepDTO[], combat: CombatDataDTO | null | undefined): StepLegality[] {
  if (!combat) return sequence.map(() => ({ legal: true, form: "", qifeng: "" }));
  let instrument = combat.form.find((f) => f.initial)?.id ?? combat.form[0]?.id ?? "";
  let qifeng = combat.qifeng?.[0] ?? "";
  const out: StepLegality[] = [];
  for (const step of sequence) {
    const key = `${step.skill.skill_id}:${step.skill.sub_id ?? 0}`;
    const def = combat.skill_combat.find((s) => `${s.skill_id}:${s.sub_id ?? 0}` === key);
    const needInstrument = def?.requires_instrument ?? null;
    const needQifeng = def?.requires_qifeng ?? null;
    const instrumentOk = !needInstrument || needInstrument === instrument;
    const qifengOk = !needQifeng || needQifeng === qifeng;
    const legal = instrumentOk && qifengOk;
    out.push({
      legal,
      form: formName(combat, instrument),
      qifeng,
      ...(instrumentOk ? {} : { requiredForm: formName(combat, needInstrument!) }),
      ...(qifengOk ? {} : { requiredQifeng: needQifeng! }),
    });
    if (legal) {
      const onCast = def?.on_cast;
      if (onCast?.switch_instrument && combat.form.some((f) => f.id === onCast.switch_instrument)) {
        instrument = onCast.switch_instrument;
      }
      if (onCast?.switch_qifeng) qifeng = onCast.switch_qifeng;
    }
  }
  return out;
}
