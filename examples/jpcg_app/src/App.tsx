import { useState, useCallback, useRef, type SetStateAction } from "react";
import { useTheme } from "./hooks/useTheme";
import { useToast } from "./hooks/useToast";
import type { FormData } from "./types";
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
import { usePersistentState } from "./hooks/usePersistentState";
import { cockpitSchema, defaultCockpit } from "./state/cockpit";
import SnapshotDiff from "./components/SnapshotDiff";
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
  const [cockpit, setCockpit] = usePersistentState(STORAGE_KEYS.cockpit, defaultCockpit, cockpitSchema);
  const [restoreRevision, setRestoreRevision] = useState(0);
  const requestRevision = useRef(0);
  const results = cockpit.calculation?.results ?? null;
  const formData = cockpit.form;
  // manualReady 表示「持久化表单已是有效快照」：只有用户编辑/已加载的存档才算，
  // 初始化的占位写入（首次挂载时先同步旧值、再异步拉取心法属性）不能提前置位，
  // 否则首次加载失败会被永久当成已初始化，下次启动跳过重新加载。
  const setFormData = useCallback((
    update: SetStateAction<FormData>,
    options?: { ready?: boolean },
  ) => {
    setCockpit((previous) => ({ ...previous,
      manualReady: options?.ready ?? true,
      form: typeof update === "function" ? update(previous.form) : update }));
  }, [setCockpit]);
  const [calculating, setCalculating] = useState(false);
  const [status, setStatus] = useState("就绪");

  const handleCalculate = useCallback(
    async (form: FormData) => {
      const revision = ++requestRevision.current;
      setCalculating(true);
      setStatus("计算中...");
      try {
        const req = toCalculateRequest(form);
        const data = await api.calculateDamage(req);
        if (revision !== requestRevision.current) return;
        setCockpit((previous) => ({ ...previous, calculation: { form, results: data } }));
        setStatus(`计算完成 — ${data.length} 个技能`);
        addToast("计算完成", "success");
      } catch (err) {
        if (revision !== requestRevision.current) return;
        setStatus("计算失败");
        addToast(String(err), "error");
      } finally {
        if (revision === requestRevision.current) setCalculating(false);
      }
    },
    [addToast, setCockpit],
  );

  const stale = !!cockpit.calculation && JSON.stringify(toCalculateRequest(formData)) !== JSON.stringify(toCalculateRequest(cockpit.calculation.form));
  const restoreBaseline = () => {
    if (!cockpit.baseline) return;
    ++requestRevision.current; // 恢复后丢弃在途计算，防止迟到响应覆盖基准。
    setCalculating(false);
    setCockpit((previous) => previous.baseline ? { ...previous, form: previous.baseline.form, calculation: previous.baseline, manualReady: true } : previous);
    setRestoreRevision((value) => value + 1);
    setStatus("已恢复钉住时的属性与结果");
  };

  return (
    <div className={styles.container}>
      <div className={styles.body}>
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
          <div className={styles.calcLayout} style={{ display: curPage === "calc" ? "" : "none" }}>
            <div className={styles.configPanel}>
              <ConfigPanel
                key={restoreRevision}
                form={formData}
                onFormChange={setFormData}
                preserveInitialForm={cockpit.manualReady}
                onCalculate={handleCalculate}
                calculating={calculating}
                addToast={addToast}
                setStatus={setStatus}
              />
            </div>
            <div className={styles.resultPanel}>
              <SnapshotDiff snapshot={cockpit.baseline} calculation={cockpit.calculation}
                stale={stale} calculating={calculating}
                onToggle={() => setCockpit((previous) => ({ ...previous, baseline: previous.baseline ? null : previous.calculation }))}
                onRestore={restoreBaseline} />
              <ResultTable
                results={results}
                baseline={cockpit.baseline?.results}
                calculating={calculating}
              />
            </div>
          </div>
          {visitedPages.has("forum") && <div className={styles.residentPage} hidden={curPage !== "forum"}><ForumPage addToast={addToast} /></div>}
          {visitedPages.has("combo") && <div className={styles.residentPage} hidden={curPage !== "combo"}>
            <ComboPage key={formData.xinfa} xinfaName={formData.xinfa} formData={formData} />
          </div>}
          {visitedPages.has("optimize") && <div className={styles.residentPage} hidden={curPage !== "optimize"}><OptimizePage formData={formData} /></div>}
          {visitedPages.has("editor") && <div className={styles.residentPage} hidden={curPage !== "editor"}><SkillEditorPage addToast={addToast} /></div>}
        </div>
      </div>
      <StatusBar message={status} />
      <Toast toasts={toasts} onRemove={removeToast} />
    </div>
  );
}
