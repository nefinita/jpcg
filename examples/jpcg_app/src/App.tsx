import { useState, useCallback } from "react";
import { useTheme } from "./hooks/useTheme";
import { useToast } from "./hooks/useToast";
import type { SkillResultDTO, FormData } from "./types";
import * as api from "./api/commands";
import ActivityBar from "./components/ActivityBar";
import ThemeToggle from "./components/ThemeToggle";
import { GITHUB_ISSUES_URL } from "./utils/constants";
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
import styles from "./App.module.css";

export default function App() {
  const { theme, toggleTheme } = useTheme();
  const { toasts, addToast, removeToast } = useToast();
  const [curPage, setCurPage] = useState<"calc" | "forum" | "combo" | "editor" | "optimize">("calc");
  const [results, setResults] = useState<SkillResultDTO[] | null>(null);
  const [calculating, setCalculating] = useState(false);
  const [status, setStatus] = useState("就绪");
  const [currentXinfa, setCurrentXinfa] = useState("mowen");
  const [formData, setFormData] = useState<FormData | null>(null);

  const handleCalculate = useCallback(
    async (form: FormData) => {
      setFormData(form);
      setCalculating(true);
      setStatus("计算中...");
      try {
        const req = toCalculateRequest(form);
        const data = await api.calculateDamage(req);
        setResults(data);
        setStatus(`计算完成 — ${data.length} 个技能`);
        addToast("计算完成", "success");
      } catch (err) {
        setStatus("计算失败");
        addToast(String(err), "error");
      } finally {
        setCalculating(false);
      }
    },
    [addToast],
  );

  return (
    <div className={styles.container}>
      <div className={styles.body}>
        <ActivityBar currentPage={curPage} onNavigate={setCurPage} />
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
                onCalculate={handleCalculate}
                calculating={calculating}
                addToast={addToast}
                setStatus={setStatus}
                onXinfaChange={setCurrentXinfa}
              />
            </div>
            <div className={styles.resultPanel}>
              <ResultTable
                results={results}
                calculating={calculating}
              />
            </div>
          </div>
          {curPage === "forum" && <ForumPage addToast={addToast} />}
          {curPage === "combo" && <ComboPage xinfaName={currentXinfa} formData={formData} />}
          {curPage === "optimize" && <OptimizePage formData={formData} />}
          {curPage === "editor" && <SkillEditorPage addToast={addToast} />}
        </div>
      </div>
      <StatusBar message={status} />
      <Toast toasts={toasts} onRemove={removeToast} />
    </div>
  );
}
