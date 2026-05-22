import { useState, useCallback } from "react";
import { useTheme } from "./hooks/useTheme";
import { useToast } from "./hooks/useToast";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import type { SkillResultDTO, FormData } from "./types";
import * as api from "./api/commands";
import ActivityBar from "./components/ActivityBar";
import ThemeToggle from "./components/ThemeToggle";
import ConfigPanel from "./components/ConfigPanel";
import ResultTable from "./components/ResultTable";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import Toast from "./components/Toast";
import styles from "./App.module.css";

export default function App() {
  const { theme, toggleTheme } = useTheme();
  const { toasts, addToast, removeToast } = useToast();
  const [sidebar, setSidebar] = useState<"forum" | "combo" | null>(null);
  const [results, setResults] = useState<SkillResultDTO[] | null>(null);
  const [calculating, setCalculating] = useState(false);
  const [status, setStatus] = useState("就绪");

  const closeSidebar = useCallback(() => setSidebar(null), []);
  const toggleSidebar = useCallback(
    (panel: "forum" | "combo") => {
      setSidebar((prev) => (prev === panel ? null : panel));
    },
    [],
  );

  useKeyboardShortcuts({ Escape: closeSidebar });

  const handleCalculate = useCallback(
    async (form: FormData) => {
      setCalculating(true);
      setStatus("计算中...");
      try {
        const req = {
          player: {
            jcsx: form.xinfa_config.xinfa_nom,
            jichu_shuxing: form.player.jichu_shuxing ?? 0,
            jichu_gongji: form.player.jichu_gongji ?? 0,
            huixin_dengji: form.player.huixin_dengji ?? 0,
            huixin_xiaoguo: form.player.huixin_xiaoguo ?? 0,
            pofang_dengji: form.player.pofang_dengji ?? 0,
            wuqi_shanghai: form.player.wuqi_shanghai ?? 0,
          },
          hostile: {
            waigong_fangyu: form.hostile.waigong_fangyu ?? 0,
            neigong_fangyu: form.hostile.neigong_fangyu ?? 0,
            yujin_dengji: form.hostile.yujin_dengji ?? 0,
            huajin_dengji: form.hostile.huajin_dengji ?? 0,
            jianshang_bili: form.hostile.jianshang_bili ?? 0,
          },
          xinfa_config: form.xinfa_config,
        };
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
        <ActivityBar
          activePanel={sidebar}
          onToggle={toggleSidebar}
        />
        <Sidebar
          panel={sidebar}
          onClose={closeSidebar}
          results={results}
        />
        <div className={styles.main}>
          <header className={styles.header}>
            <div className={styles.logo}>剑心 PVP 计算器</div>
            <ThemeToggle theme={theme} onToggle={toggleTheme} />
          </header>
          <div className={styles.content}>
            <div className={styles.configPanel}>
              <ConfigPanel
                onCalculate={handleCalculate}
                calculating={calculating}
                addToast={addToast}
                setStatus={setStatus}
              />
            </div>
            <div className={styles.resultPanel}>
              <ResultTable
                results={results}
                calculating={calculating}
              />
            </div>
          </div>
        </div>
      </div>
      <StatusBar message={status} />
      <Toast toasts={toasts} onRemove={removeToast} />
    </div>
  );
}
