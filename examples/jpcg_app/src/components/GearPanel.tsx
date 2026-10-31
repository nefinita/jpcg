import type { GearCatalog, GearChoice, GearComposition, GearLoadout } from "../types/gear";
import { GEAR_SLOTS, emptyGearStats } from "../types/gear";
import { choiceKey, slotNames, sourceNames, greenNames, statNames, MOWEN_ATTRIBUTE_RULES, supportsAttributeRules } from "../state/gear";
import styles from "./GearPanel.module.css";

const basicFields = ["tizhi", "liliang", "shenfa", "gengu", "yuanqi"] as const;

export default function GearPanel({ catalog, loadout, composition, onChange, onReset, onExport, onImport, pending }: {
  catalog: GearCatalog; loadout: GearLoadout; composition?: GearComposition;
  onChange: (loadout: GearLoadout) => void; onReset: () => void; onExport: () => void;
  onImport: (text: string) => Promise<void>; pending: boolean;
}) {
  const stale = loadout.revision !== catalog.version.revision || loadout.value_set !== catalog.version.value_set;
  const supportedRules = supportsAttributeRules(loadout);
  const panel = composition?.composed.panel;
  const legacyHasBasics = !loadout.attribute_rules && basicFields.some((key) => (loadout.base[key] ?? 0) > 0);
  const change = (choice: GearChoice) => onChange({ ...loadout, choices: loadout.choices.map((c) => c.slot === choice.slot ? choice : c) });
  return <section className={styles.panel} aria-label="配装">
    <header className={styles.header}>
      <div><h2>当前配装</h2><p>{catalog.version.sample ? "合成测试样例 · 非游戏装备数值" : `赛季 ${catalog.version.season}`} · {catalog.version.revision}</p></div>
      <div className={styles.actions}>
        <button onClick={onReset}>重置配装</button>
        <button onClick={onExport} disabled={stale}>导出配装</button>
        <label>导入配装<input type="file" accept=".toml" onChange={async (e) => {
          const file = e.target.files?.[0]; if (file) await onImport(await file.text()); e.target.value = "";
        }} /></label>
      </div>
    </header>
    {stale && <div role="alert" className={styles.notice}>保存的配装与当前模板版本不一致。<button onClick={onReset}>按当前模板重置</button></div>}
    <div className={styles.base}>
      <label>属性换算规则 <select aria-label="属性换算规则" value={loadout.attribute_rules ?? ""} disabled={stale} onChange={(event) => onChange({ ...loadout, attribute_rules: event.target.value || null })}>
        <option value="">旧版合成（不换算体质与基础属性会心）</option>
        {supportedRules && <option value={MOWEN_ATTRIBUTE_RULES}>130 级莫问 · 2026-08-14 参考规则</option>}
        {loadout.attribute_rules && (!supportedRules || loadout.attribute_rules !== MOWEN_ATTRIBUTE_RULES) && <option value={loadout.attribute_rules}>已保存规则：{loadout.attribute_rules}（当前范围不支持）</option>}
      </select></label>
      {loadout.attribute_rules ? <p>参考规则已核对来源，尚未经真实游戏面板验证；仅适用所选版本及心法。</p> : <p>体质及五维属性换算需选择对应规则；旧配装保持原有计算方式。</p>}
      {legacyHasBasics && <p role="alert">旧版合成不支持这些五维输入。请重新选择换算规则，或在下方将已填写的体质、力道、身法、根骨、元气清零后再计算。</p>}
    </div>
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
      <p>{loadout.attribute_rules ? "填写主属性及心法换算前的原始基础点数，不可直接抄裸装最终面板；此处不会自动补入等级或心法基础值。" : "仅填写脱下装备后的基础点数，避免重复叠加。"}</p>
      {loadout.attribute_rules && <p>主属性会与对应五维字段相加，例如莫问的主属性加到根骨；同一来源只填写一次。</p>}
      <div className={styles.baseFields}>{Object.entries({ ...emptyGearStats(), ...loadout.base }).map(([k, n]) => <label key={k}>{statNames[k as keyof typeof statNames]}
        <input type="number" min="0" max="4294967295" step="1" value={n ?? 0} disabled={!loadout.attribute_rules && (basicFields as readonly string[]).includes(k) && !n} onChange={(e) => { const value = Number(e.target.value); if (Number.isInteger(value) && value >= 0 && value <= 0xffffffff) onChange({ ...loadout, base: { ...loadout.base, [k]: value } }); }} />
      </label>)}</div>
    </details>
    {composition && <div className={styles.summary} aria-live="polite">
      <strong>{pending ? "正在更新合成结果…" : "合成属性"}</strong>
      <p>{Object.entries(composition.composed.stats).filter(([, n]) => n > 0).map(([k, n]) => `${statNames[k as keyof typeof statNames]} ${n}`).join(" · ")}</p>
      {panel && <div aria-label="换算后玩家属性">
        <strong>换算后玩家属性（不含战斗增益）</strong>
        <p>气血 {panel.qixue} · 攻击 {panel.gongji}（基础 {panel.gongji_base} + 转化 {panel.gongji_extra}） · 会心等级 {panel.huixin} · 破防等级 {panel.pofang}</p>
        <p>体质 {panel.tizhi} · 力道 {panel.liliang} · 身法 {panel.shenfa} · 根骨 {panel.gengu} · 元气 {panel.yuanqi}</p>
        {!panel.verified_in_game && <p>此结果尚未通过真实游戏面板验证。</p>}
      </div>}
      <p>已激活套装阈值：{composition.composed.active_bonuses.map((b) => `${b.set} ${b.threshold}件${b.tier === "upgraded" ? "升级" : "基础"}`).join(" · ") || "无"}</p>
      {composition.composed.cooldowns.length > 0 && <p>4 件套 CD 尚未应用到时序计算；此处伤害仅体现属性加成。</p>}
      {composition.composed.stats.jiasu > 0 && <p>加速点数已合成，时序收益待战斗时序功能支持。</p>}
      {composition.composed.warnings.map((warning) => <p key={warning}>{warning}</p>)}
    </div>}
  </section>;
}
