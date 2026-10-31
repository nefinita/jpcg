import type { GearCatalog, GearChoice, GearComposition, GearLoadout } from "../types/gear";
import { GEAR_SLOTS } from "../types/gear";
import { choiceKey, slotNames, sourceNames, greenNames, statNames } from "../state/gear";
import styles from "./GearPanel.module.css";

export default function GearPanel({ catalog, loadout, composition, onChange, onReset, onExport, onImport, pending }: {
  catalog: GearCatalog; loadout: GearLoadout; composition?: GearComposition;
  onChange: (loadout: GearLoadout) => void; onReset: () => void; onExport: () => void;
  onImport: (file: File) => Promise<void>; pending: boolean;
}) {
  const stale = loadout.revision !== catalog.version.revision || loadout.value_set !== catalog.version.value_set;
  const change = (choice: GearChoice) => onChange({ ...loadout, choices: loadout.choices.map((c) => c.slot === choice.slot ? choice : c) });
  return <section className={styles.panel} aria-label="配装">
    <header className={styles.header}>
      <div><h2>当前配装</h2><p>{catalog.version.sample ? "合成测试样例 · 非游戏装备数值" : `赛季 ${catalog.version.season}`} · {catalog.version.revision}</p></div>
      <div className={styles.actions}>
        <button onClick={onReset}>重置配装</button>
        <button onClick={onExport} disabled={stale}>导出配装</button>
        <label>导入配装<input type="file" accept=".toml" onChange={(e) => {
          const file = e.currentTarget.files?.[0]; e.currentTarget.value = "";
          if (file) void onImport(file);
        }} /></label>
      </div>
    </header>
    {stale && <div role="alert" className={styles.notice}>保存的配装与当前模板版本不一致。<button onClick={onReset}>按当前模板重置</button></div>}
    <div className={styles.cards}>
      {GEAR_SLOTS.map((slot) => {
        const candidates = catalog.templates.filter((t) => t.school === loadout.school && t.choice.slot === slot);
        const selected = loadout.choices.find((c) => c.slot === slot);
        const template = candidates.find((t) => selected && choiceKey(t.choice) === choiceKey(selected));
        return <article className={styles.card} key={slot} aria-label={slotNames[slot]}>
          <h3>{slotNames[slot]}</h3>
          <strong>{template ? `${sourceNames[template.choice.source]} · ${greenNames[template.green]}` : "模板不可用"}</strong>
          <div className={styles.choices}>
            {candidates.map((t) => <button key={choiceKey(t.choice)} disabled={stale} aria-pressed={!!selected && choiceKey(t.choice) === choiceKey(selected)} onClick={() => change(t.choice)}>
              {sourceNames[t.choice.source]} · {t.choice.tier === "base" ? "基础" : "升级"}{t.choice.variant > 0 ? ` · 属性套${t.choice.variant}` : ""}
            </button>)}
          </div>
          {template && <>
            <p>{Object.entries(template.stats).filter(([, n]) => n > 0).slice(0, 3).map(([k, n]) => `${statNames[k as keyof typeof statNames]} ${n}`).join(" · ")}</p>
            <details><summary>属性详情{template.zhuangfen != null ? ` · 参考装分 ${template.zhuangfen}` : ""}</summary>
              <dl>{Object.entries(template.stats).map(([k, n]) => <div key={k}><dt>{statNames[k as keyof typeof statNames]}</dt><dd>{n}</dd></div>)}</dl>
            </details>
          </>}
        </article>;
      })}
    </div>
    <details className={styles.base}><summary>裸装基础值（不含装备与套装）</summary>
      <p>仅填写脱下装备后的基础点数，避免重复叠加。</p>
      <div className={styles.baseFields}>{Object.entries(loadout.base).map(([k, n]) => <label key={k}>{statNames[k as keyof typeof statNames]}
        <input type="number" min="0" step="1" value={n} onChange={(e) => { const value = Number(e.target.value); if (Number.isInteger(value) && value >= 0) onChange({ ...loadout, base: { ...loadout.base, [k]: value } }); }} />
      </label>)}</div>
    </details>
    {composition && <div className={styles.summary} aria-live="polite">
      <strong>{pending ? "正在更新合成结果…" : "合成属性"}</strong>
      <p>{Object.entries(composition.composed.stats).filter(([, n]) => n > 0).map(([k, n]) => `${statNames[k as keyof typeof statNames]} ${n}`).join(" · ")}</p>
      <p>已激活套装阈值：{composition.composed.active_bonuses.map((b) => `${b.set} ${b.threshold}件${b.tier === "upgraded" ? "升级" : "基础"}`).join(" · ") || "无"}</p>
      {composition.composed.cooldowns.length > 0 && <p>4 件套 CD 尚未应用到时序计算；此处伤害仅体现属性加成。</p>}
      {composition.composed.stats.jiasu > 0 && <p>加速点数已合成，时序收益待战斗时序功能支持。</p>}
      {composition.composed.warnings.map((warning) => <p key={warning}>{warning}</p>)}
    </div>}
  </section>;
}
