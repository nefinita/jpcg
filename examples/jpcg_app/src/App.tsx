import { useState, useCallback, useRef, useEffect, type SetStateAction } from "react";
import { useTheme } from "./hooks/useTheme";
import { useToast } from "./hooks/useToast";
import type { FormData, ComboResultDTO } from "./types";
import * as api from "./api/commands";
import ActivityBar from "./components/ActivityBar";
import ThemeToggle from "./components/ThemeToggle";
import { GITHUB_ISSUES_URL, STORAGE_KEYS } from "./utils/constants";
import ConfigPanel from "./components/ConfigPanel";
import ResultTable from "./components/ResultTable";
import ForumPage from "./components/ForumPage";
import ComboPage from "./components/ComboPage";
import OptimizePage from "./components/OptimizePage";
import SkillEditorPage from "./components/SkillEditorPage";
import StatusBar from "./components/StatusBar";
import Toast from "./components/Toast";
import { IconBug, IconCalc } from "./components/icons";
import { toCalculateRequest } from "./utils/normalize";
import { usePersistentState, writePreference, readPreference } from "./hooks/usePersistentState";
import { cockpitSchema, defaultCockpit, comboDraftSchema, defaultComboDraft } from "./state/cockpit";
import SnapshotDiff from "./components/SnapshotDiff";
import type { GearCatalog, GearLoadout } from "./types/gear";
import { initialLoadout, gearInputKey } from "./state/gear";
import CockpitCombo from "./components/CockpitCombo";
import GearPanel from "./components/GearPanel";
import styles from "./App.module.css";

export default function App() {
  const { theme, toggleTheme } = useTheme();
  const { toasts, addToast, removeToast } = useToast();
  const [curPage, setCurPage] = useState<"calc" | "forum" | "combo" | "editor" | "optimize">("calc");
  const [visitedPages, setVisitedPages] = useState(() => new Set(["calc"]));
  const navigate = (page: typeof curPage) => {
    setCurPage(page);
    setVisitedPages((previous) => previous.has(page) ? previous : new Set([...previous, page]));
  };
  const freshCockpit = useRef(readPreference(STORAGE_KEYS.cockpit) === null);
  const [contextReady, setContextReady] = useState(false);
  const [cockpit, setCockpit] = usePersistentState(STORAGE_KEYS.cockpit, defaultCockpit, cockpitSchema);
  const [restoreRevision, setRestoreRevision] = useState(0);
  const requestRevision = useRef(0);
  const actionRevision = useRef(0);
  const [calculating, setCalculating] = useState(false);
  const [status, setStatus] = useState("就绪");
  const [catalog, setCatalog] = useState<GearCatalog | null>(null);
  const [gearError, setGearError] = useState("");
  const [catalogAttempt, setCatalogAttempt] = useState(0);
  const formData = cockpit.form;
  const [comboDraft, setComboDraft] = usePersistentState(`${STORAGE_KEYS.comboDraft}:${encodeURIComponent(formData.xinfa)}`, defaultComboDraft, comboDraftSchema);
  const [comboBusy, setComboBusy] = useState(false);
  const saveComboResult = useCallback((inputKey: string, result: ComboResultDTO) => {
    setCockpit((previous) => ({ ...previous, combo: { inputKey, result } }));
  }, [setCockpit]);
  const loadout = cockpit.loadouts[formData.xinfa];
  const results = cockpit.calculation?.results ?? null;
  const gearMode = cockpit.mode === "gear";
  // manualReady 表示「持久化表单已是有效快照」：只有用户编辑/已加载的存档才算，
  // 初始化的占位写入（首次挂载时先同步旧值、再异步拉取心法属性）不能提前置位，
  // 否则首次加载失败会被永久当成已初始化，下次启动跳过重新加载。
  const setFormData = useCallback((
    update: SetStateAction<FormData>,
    options?: { ready?: boolean },
  ) => {
    ++actionRevision.current;
    setCockpit((previous) => ({ ...previous,
      manualReady: options?.ready ?? true,
      form: typeof update === "function" ? update(previous.form) : update }));
  }, [setCockpit]);

  useEffect(() => {
    let cancelled = false;
    api.gearCatalog().then((value) => { if (!cancelled) { setCatalog(value); setGearError(""); } })
      .catch((err) => { if (!cancelled) setGearError(String(err)); });
    return () => { cancelled = true; };
  }, [catalogAttempt]);

  useEffect(() => {
    if (!catalog || loadout) return;
    const next = initialLoadout(catalog, formData.xinfa);
    if (next) setCockpit((previous) => ({ ...previous, loadouts: { ...previous.loadouts, [next.school]: next } }));
  }, [catalog, loadout, formData.xinfa, setCockpit]);

  const desiredGearKey = loadout ? gearInputKey(formData, loadout) : "";
  const compatible = contextReady && !!catalog && !!loadout && loadout.school === formData.xinfa && loadout.revision === catalog.version.revision
    && loadout.value_set === catalog.version.value_set && formData.value_set === loadout.value_set;
  const stale = !!cockpit.calculation && (gearMode
    ? cockpit.calculation.mode !== "gear" || cockpit.calculation.inputKey !== desiredGearKey
    : cockpit.calculation.mode === "gear" || JSON.stringify(toCalculateRequest(formData)) !== JSON.stringify(toCalculateRequest(cockpit.calculation.form)));

  useEffect(() => {
    if (!gearMode || curPage !== "calc" || !compatible || !loadout) { setCalculating(false); return; }
    if (cockpit.calculation?.mode === "gear" && cockpit.calculation.inputKey === desiredGearKey) return;
    const revision = ++requestRevision.current;
    const timer = window.setTimeout(async () => {
      setCalculating(true); setGearError(""); setStatus("合成配装并计算中…");
      try {
        const composition = await api.composeGear(loadout);
        if (revision !== requestRevision.current) return;
        const { jcsx, ...player } = composition.player;
        const effectiveForm: FormData = { ...formData, player,
          xinfa_config: { ...formData.xinfa_config, xinfa_nom: jcsx } };
        const data = await api.calculateDamage(toCalculateRequest(effectiveForm));
        if (revision !== requestRevision.current) return;
        setCockpit((previous) => ({ ...previous, calculation: { mode: "gear", form: effectiveForm,
          manualForm: formData, loadout, composition, inputKey: desiredGearKey, results: data } }));
        setStatus(`配装已更新 — ${data.length} 个技能`);
      } catch (err) {
        if (revision === requestRevision.current) { setGearError(String(err)); setStatus("配装计算失败"); }
      } finally { if (revision === requestRevision.current) setCalculating(false); }
    }, 125);
    return () => { window.clearTimeout(timer); ++requestRevision.current; };
    // desiredGearKey 包含完整上下文与配装；结果入库不重启同一输入的计算。
  }, [desiredGearKey, compatible, gearMode, curPage, catalog, setCockpit]);

  const handleCalculate = useCallback(async (form: FormData) => {
    const revision = ++requestRevision.current;
    setCalculating(true); setStatus("计算中...");
    try {
      const data = await api.calculateDamage(toCalculateRequest(form));
      if (revision !== requestRevision.current) return;
      setCockpit((previous) => ({ ...previous, calculation: { mode: "manual", form, manualForm: previous.form, results: data } }));
      setStatus(`计算完成 — ${data.length} 个技能`); addToast("计算完成", "success");
    } catch (err) { if (revision === requestRevision.current) { setStatus("计算失败"); addToast(String(err), "error"); } }
    finally { if (revision === requestRevision.current) setCalculating(false); }
  }, [addToast, setCockpit]);

  const restoreBaseline = () => {
    if (!cockpit.baseline) return;
    ++requestRevision.current; ++actionRevision.current; setCalculating(false); setGearError("");
    const base = cockpit.baseline;
    if (base.comboDraft) {
      if (base.form.xinfa === formData.xinfa) setComboDraft(base.comboDraft);
      else writePreference(`${STORAGE_KEYS.comboDraft}:${encodeURIComponent(base.form.xinfa)}`, JSON.stringify({ version: comboDraftSchema.version, data: base.comboDraft }));
    }
    setCockpit((previous) => {
      const base = previous.baseline; if (!base) return previous;
      return { ...previous, mode: base.mode ?? "manual", form: base.manualForm ?? base.form,
        loadouts: base.loadout ? { ...previous.loadouts, [base.loadout.school]: base.loadout } : previous.loadouts,
        calculation: base, manualReady: true,
        combo: base.comboResult && base.comboDraft ? { inputKey: JSON.stringify({ request: toCalculateRequest(base.form), sequence: base.comboDraft.sequence }), result: base.comboResult } : undefined };
    });
    setRestoreRevision((value) => value + 1); setStatus("已恢复钉住时的配装、属性与结果");
  };
  const adoptValueSet = async (nextLoadout?: GearLoadout) => {
    const selected = nextLoadout ?? loadout; if (!selected) return;
    const action = ++actionRevision.current;
    try {
      const vs = await api.resolveValueSet(selected.value_set);
      if (!vs.available || vs.id !== selected.value_set || !vs.coefficient) throw new Error("模板对应数值集不可用，请先更新数据");
      const config = selected.school !== formData.xinfa ? await api.loadProfessionConfig(selected.school) : null;
      if (selected.school !== formData.xinfa && !config) throw new Error("配装心法的数据不可用，请先更新技能数据");
      if (action !== actionRevision.current) return;
      setCockpit((previous) => ({ ...previous, mode: "gear", manualReady: true,
        loadouts: { ...previous.loadouts, [selected.school]: selected },
        form: { ...previous.form, xinfa: selected.school, value_set: selected.value_set,
          coefficient: previous.form.value_set === selected.value_set ? previous.form.coefficient : { ...vs.coefficient! },
          xinfa_config: config ? { profession: selected.school, xinfa_name: String(config.xinfa_name ?? selected.school),
            xinfa_nom: String(config.xinfa_nom ?? previous.form.xinfa_config.xinfa_nom), atk_up: Number(config.atk_up) || 0,
            pofang_up: Number(config.pofang_up) || 0, huixin_up: Number(config.huixin_up) || 0 }
            : previous.form.xinfa_config } }));
      setRestoreRevision((value) => value + 1); setGearError("");
    } catch (err) { addToast(String(err), "error"); }
  };
  useEffect(() => {
    if (!freshCockpit.current || !gearMode || !loadout || !contextReady) return;
    freshCockpit.current = false;
    void adoptValueSet();
  }, [gearMode, loadout, contextReady]);
  const changeLoadout = (next: GearLoadout) => { ++actionRevision.current; setCockpit((previous) => ({ ...previous, loadouts: { ...previous.loadouts, [next.school]: next } })); };
  const exportLoadout = async () => {
    if (!loadout) return;
    try {
      const text = await api.exportGear(loadout);
      const url = URL.createObjectURL(new Blob([text], { type: "application/toml" }));
      const link = document.createElement("a"); link.href = url; link.download = `${loadout.school}_gear.toml`; link.click(); URL.revokeObjectURL(url);
    } catch (err) { addToast(String(err), "error"); }
  };
  const importLoadout = async (text: string) => {
    try { const value = await api.importGear(text); await adoptValueSet(value.composed.loadout); }
    catch (err) { addToast(String(err), "error"); }
  };
  const composition = cockpit.calculation?.mode === "gear" && cockpit.calculation.loadout?.school === formData.xinfa ? cockpit.calculation.composition : undefined;
  const activeForm = gearMode ? (compatible && !stale && cockpit.calculation?.mode === "gear" ? cockpit.calculation.form : null) : formData;
  const comboKey = JSON.stringify({ request: activeForm ? toCalculateRequest(activeForm) : null, sequence: comboDraft.sequence });
  const resultPanel = <div className={styles.resultPanel}>
    <SnapshotDiff snapshot={cockpit.baseline} calculation={cockpit.calculation} stale={stale} calculating={calculating || comboBusy}
      onToggle={() => setCockpit((previous) => ({ ...previous, baseline: previous.baseline ? null : previous.calculation ? { ...previous.calculation, comboDraft, comboResult: previous.combo?.inputKey === comboKey ? previous.combo.result : undefined } : null }))} onRestore={restoreBaseline} />
    <CockpitCombo form={activeForm} draft={comboDraft} inputKey={comboKey} saved={cockpit.combo} baseline={cockpit.baseline?.comboResult}
      active={curPage === "calc"} onResult={saveComboResult} onBusy={setComboBusy} />
    <ResultTable results={results} baseline={cockpit.baseline?.results} calculating={calculating} />
  </div>;
  const configPanel = <ConfigPanel key={restoreRevision} form={formData} onFormChange={setFormData}
    preserveInitialForm={cockpit.manualReady} manualMode={!gearMode} onReady={setContextReady}
    playerPreview={composition ? { ...composition.player } : undefined}
    onCalculate={handleCalculate} calculating={calculating} addToast={addToast} setStatus={setStatus} />;

  return <div className={styles.container}><div className={styles.body}>
    <ActivityBar currentPage={curPage} onNavigate={navigate} />
    <div className={styles.main}>
          <header className={styles.header}>
            <div className={styles.logo}>
              <span className={styles.logoMark}><IconCalc size={17} /></span>
              剑网3PVP计算器（JPCG）
            </div>
            <div className={styles.headerActions}>
              <button
                className={styles.headerBtn}
                onClick={() => window.open(GITHUB_ISSUES_URL, "_blank")}
                title="反馈 / Issues"
              >
                <IconBug size={16} />
              </button>
              <ThemeToggle theme={theme} onToggle={toggleTheme} />
            </div>
          </header>

      <div className={`${styles.calcLayout} ${gearMode ? styles.gearLayout : ""}`} style={{ display: curPage === "calc" ? "" : "none" }}>
        <div className={styles.modeBar} role="group" aria-label="输入模式">
          <button aria-pressed={gearMode} onClick={() => { if (gearMode) return; ++requestRevision.current; ++actionRevision.current; setCockpit((p) => ({ ...p, mode: "gear" })); }}>配装驱动</button>
          <button aria-pressed={!gearMode} onClick={() => { if (!gearMode) return; ++requestRevision.current; ++actionRevision.current; setCockpit((p) => ({ ...p, mode: "manual" })); }}>手动属性</button>
        </div>
        {gearMode ? <>
          <details className={styles.contextPanel}><summary>{formData.xinfa_config.xinfa_name} · {formData.value_set ?? "默认数值集"} · 目标血量 {formData.hostile.target_hp || 0} · 编辑计算环境</summary>{configPanel}</details>
          {gearError && <div role="alert" className={styles.gearAlert}>{gearError}<button onClick={() => setCatalogAttempt((n) => n + 1)}>重新读取模板</button></div>}
          {!catalog && !gearError && <p role="status">正在读取配装模板…</p>}
          {catalog && !loadout && <p>当前心法没有完整的 12 部位模板，请选择其他心法或切换手动属性。</p>}
          {catalog && loadout && !compatible && <div className={styles.gearAlert}>配装需要匹配模板版本与数值集。<button onClick={() => void adoptValueSet()}>使用模板数值集 {loadout.value_set}</button></div>}
          <div className={styles.gearBody}>
            {catalog && loadout && <GearPanel catalog={catalog} loadout={loadout} composition={composition} pending={stale || calculating}
              onChange={changeLoadout} onReset={() => { const next = initialLoadout(catalog, formData.xinfa); if (next) changeLoadout(next); }}
              onExport={() => void exportLoadout()} onImport={importLoadout} />}
            {resultPanel}
          </div>
        </> : <div className={styles.manualBody}><div className={styles.configPanel}>{configPanel}</div>{resultPanel}</div>}
      </div>
      {visitedPages.has("forum") && <div className={styles.residentPage} hidden={curPage !== "forum"}><ForumPage addToast={addToast} /></div>}
      {visitedPages.has("combo") && <div className={styles.residentPage} hidden={curPage !== "combo"}><ComboPage key={formData.xinfa} xinfaName={formData.xinfa} formData={activeForm} draft={comboDraft} onDraftChange={setComboDraft} configurationOnly /></div>}
      {visitedPages.has("optimize") && <div className={styles.residentPage} hidden={curPage !== "optimize"}><OptimizePage formData={activeForm} /></div>}
      {visitedPages.has("editor") && <div className={styles.residentPage} hidden={curPage !== "editor"}><SkillEditorPage addToast={addToast} /></div>}
    </div>
  </div><StatusBar message={status} /><Toast toasts={toasts} onRemove={removeToast} /></div>;
}
