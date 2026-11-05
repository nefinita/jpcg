import type { CombatDataDTO, ComboStepDTO } from "../types";

export interface StepLegality {
  /** 该动作在当前形态下是否可用。 */
  legal: boolean;
  /** 执行到这一步之前所处的形态名。 */
  form: string;
  /** 需要但当前不具备的形态名（非法时给出）。 */
  requiredForm?: string;
}

const formName = (combat: CombatDataDTO, id: string): string => combat.form.find((f) => f.id === id)?.name ?? id;

/** 按动作顺序模拟形态切换；形态不符的技能标记为非法。 */
export function checkFormLegality(sequence: ComboStepDTO[], combat: CombatDataDTO | null | undefined): StepLegality[] {
  if (!combat || combat.form.length === 0) return sequence.map(() => ({ legal: true, form: "" }));
  let current = combat.form.find((f) => f.initial)?.id ?? combat.form[0].id;
  const out: StepLegality[] = [];
  for (const step of sequence) {
    const key = `${step.skill.skill_id}:${step.skill.sub_id ?? 0}`;
    const def = combat.skill_combat.find((s) => `${s.skill_id}:${s.sub_id ?? 0}` === key);
    const required = def?.form ?? null;
    const legal = !required || required === current;
    out.push({ legal, form: formName(combat, current), ...(legal ? {} : { requiredForm: formName(combat, required!) }) });
    if (legal && def?.on_cast?.switch_form && combat.form.some((f) => f.id === def.on_cast!.switch_form)) {
      current = def.on_cast!.switch_form!;
    }
  }
  return out;
}
