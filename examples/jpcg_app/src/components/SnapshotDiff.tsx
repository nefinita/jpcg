import type { CalculationSnapshot } from "../state/cockpit";
import styles from "./SnapshotDiff.module.css";

export default function SnapshotDiff({ snapshot, calculation, stale, calculating, onToggle, onRestore }: {
  snapshot: CalculationSnapshot | null;
  calculation: CalculationSnapshot | null;
  stale: boolean;
  calculating: boolean;
  onToggle: () => void;
  onRestore: () => void;
}) {
  return <section className={styles.bar} aria-label="结果基准">
    <div>
      <strong>计算驾驶台</strong>
      {calculation && <p>结果：{calculation.form.xinfa_config.xinfa_name} · {calculation.form.value_set ?? "默认数值集"}</p>}
      {stale && <p role="status">属性已修改，请重新计算以更新结果。</p>}
      {snapshot && <p>基准：{snapshot.form.xinfa_config.xinfa_name} · {snapshot.form.value_set ?? "默认数值集"}（差值按各次计算的上下文比较）</p>}
    </div>
    <div className={styles.actions}>
      <button onClick={onToggle} aria-pressed={!!snapshot}
        disabled={!snapshot && (!calculation?.results.length || stale || calculating)}>
        {snapshot ? "取消钉住" : "钉住基准"}
      </button>
      {snapshot && <button onClick={onRestore}>恢复基准</button>}
    </div>
  </section>;
}
