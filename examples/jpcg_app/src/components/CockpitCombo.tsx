import { useEffect, useState } from "react";
import type { ComboDraft } from "../state/cockpit";
import type { ComboResultDTO, FormData } from "../types";
import * as api from "../api/commands";
import { toCalculateRequest } from "../utils/normalize";
import styles from "./SnapshotDiff.module.css";

export default function CockpitCombo({ form, draft, inputKey, saved, baseline, active, onResult, onBusy }: {
  form: FormData | null; draft: ComboDraft; inputKey: string;
  saved?: { inputKey: string; result: ComboResultDTO }; baseline?: ComboResultDTO; active: boolean;
  onResult: (key: string, result: ComboResultDTO) => void; onBusy: (busy: boolean) => void;
}) {
  const [error, setError] = useState("");
  useEffect(() => {
    if (!active || !form || !draft.sequence.length || saved?.inputKey === inputKey) { onBusy(false); return; }
    let cancelled = false;
    onBusy(true); setError("");
    const timer = window.setTimeout(async () => {
      try {
        const request = toCalculateRequest(form);
        const result = await api.calculateCombo(draft.sequence, request.player, request.hostile, request.xinfa_config, request.buff, request.coefficient, request.value_set);
        if (!cancelled) onResult(inputKey, result);
      } catch (err) { if (!cancelled) setError(String(err)); }
      finally { if (!cancelled) onBusy(false); }
    }, 125);
    return () => { cancelled = true; window.clearTimeout(timer); onBusy(false); };
  }, [inputKey, active, onResult, onBusy]);
  if (!draft.sequence.length) return <section className={styles.bar}>在排轴器编辑连招后，驾驶台将自动计算击杀率。</section>;
  const current = saved?.inputKey === inputKey ? saved.result : null;
  const difference = current && baseline ? (current.final_kill_prob - baseline.final_kill_prob) * 100 : null;
  return <section className={styles.bar} aria-label="连招结果">
    <div><strong>连招 · {draft.sequence.length} 步</strong>
      {!form && <p>先完成当前属性计算。</p>}
      {error && <p role="alert">{error}</p>}
      {form && !current && !error && <p role="status">正在计算连招…</p>}
      {current && <p>期望伤害 {current.total_expected_damage_wan.toFixed(1)} 万 · 击杀率 {(current.final_kill_prob * 100).toFixed(1)}%
        {difference !== null && ` · Δ ${difference >= 0 ? "+" : ""}${difference.toFixed(1)} 个百分点`}</p>}
    </div>
  </section>;
}
