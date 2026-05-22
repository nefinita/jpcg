import { useState, useCallback, useEffect, useRef } from "react";
import type { FormData, UpdateProgressEvent, UpdateCheckResult } from "../types";
import { XINFA_LIST, PLAYER_FIELDS, HOSTILE_FIELDS, STORAGE_KEYS } from "../utils/constants";
import * as api from "../api/commands";
import styles from "./ConfigPanel.module.css";

interface Props {
  onCalculate: (form: FormData) => void;
  calculating: boolean;
  addToast: (msg: string, type?: "success" | "error" | "warning" | "info") => void;
  setStatus: (msg: string) => void;
}

const defaultForm = (): FormData => ({
  xinfa: "mowen",
  player: Object.fromEntries(PLAYER_FIELDS.map((f) => [f.id, 0])) as Record<string, number>,
  hostile: Object.fromEntries(HOSTILE_FIELDS.map((f) => [f.id, 0])) as Record<string, number>,
  xinfa_config: {
    xinfa_name: "莫问",
    xinfa_nom: "gengu",
    atk_up: 0,
    pofang_up: 0,
    huixin_up: 0,
  },
});

export default function ConfigPanel({ onCalculate, calculating, addToast, setStatus }: Props) {
  const [form, setForm] = useState<FormData>(() => {
    const last = typeof localStorage !== "undefined"
      ? localStorage.getItem(STORAGE_KEYS.lastXinfa)
      : null;
    const defaultXinfa = last || "mowen";
    const entry = XINFA_LIST.find((x) => x.value === defaultXinfa) || XINFA_LIST[0];
    return { ...defaultForm(), xinfa: defaultXinfa, xinfa_config: { ...defaultForm().xinfa_config, xinfa_name: entry.label } };
  });

  // Update progress state
  const [updating, setUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [updateMessage, setUpdateMessage] = useState("");
  const [updateCheckResult, setUpdateCheckResult] = useState<UpdateCheckResult | null>(null);

  const xinfaEntry = XINFA_LIST.find((x) => x.value === form.xinfa);
  const defaultXinfa = XINFA_LIST.find((x) => x.default) || XINFA_LIST[0];

  const handleXinfaChange = useCallback((value: string) => {
    const entry = XINFA_LIST.find((x) => x.value === value) || defaultXinfa;
    localStorage.setItem(STORAGE_KEYS.lastXinfa, value);
    setForm((prev): FormData => ({
      ...prev,
      xinfa: value,
      xinfa_config: { ...prev.xinfa_config, xinfa_name: entry.label },
    }));
    // Load profession config
    api.loadProfessionConfig(value).then((cfg) => {
      if (cfg) {
        setForm((prev): FormData => ({
          ...prev,
          xinfa_config: {
            xinfa_name: String(cfg.xinfa_name),
            xinfa_nom: String(cfg.xinfa_nom),
            atk_up: Number(cfg.atk_up) || 0,
            pofang_up: Number(cfg.pofang_up) || 0,
            huixin_up: Number(cfg.huixin_up) || 0,
          },
        }));
      }
    }).catch(() => {});
  }, [defaultXinfa]);

  const updateField = useCallback(
    (section: "player" | "hostile", id: string, value: string) => {
      const num = value === "" ? 0 : Number(value);
      setForm((prev) => ({
        ...prev,
        [section]: { ...prev[section], [id]: isNaN(num) ? 0 : num },
      }));
    },
    [],
  );

  const handleCalculate = useCallback(() => {
    onCalculate(form);
  }, [form, onCalculate]);

  const handleSave = useCallback(async () => {
    try {
      const req = {
        player: form.player as Record<string, unknown>,
        hostile: form.hostile as Record<string, unknown>,
        xinfa_config: form.xinfa_config,
      };
      await api.saveConfig({
        player: req.player as never,
        hostile: req.hostile as never,
        xinfa_config: req.xinfa_config,
      });
      addToast("配置已保存", "success");
    } catch (err) {
      addToast(String(err), "error");
    }
  }, [form, addToast]);

  const handleLoad = useCallback(async () => {
    try {
      const cfg = await api.loadConfig();
      if (!cfg) {
        addToast("没有已保存的配置", "warning");
        return;
      }
      const entry = XINFA_LIST.find((x) => x.label === cfg.xinfa_config.xinfa_name);
      const xinfaVal = entry?.value || "mowen";
      localStorage.setItem(STORAGE_KEYS.lastXinfa, xinfaVal);
      setForm({
        xinfa: xinfaVal,
        player: {
          jichu_shuxing: cfg.player.jichu_shuxing,
          jichu_gongji: cfg.player.jichu_gongji,
          huixin_dengji: cfg.player.huixin_dengji,
          huixin_xiaoguo: cfg.player.huixin_xiaoguo,
          pofang_dengji: cfg.player.pofang_dengji,
          wuqi_shanghai: cfg.player.wuqi_shanghai,
        },
        hostile: {
          waigong_fangyu: cfg.hostile.waigong_fangyu,
          neigong_fangyu: cfg.hostile.neigong_fangyu,
          yujin_dengji: cfg.hostile.yujin_dengji,
          huajin_dengji: cfg.hostile.huajin_dengji,
          jianshang_bili: cfg.hostile.jianshang_bili,
        },
        xinfa_config: cfg.xinfa_config,
      });
      addToast("配置已加载", "success");
    } catch (err) {
      addToast(String(err), "error");
    }
  }, [addToast]);

  const handleClear = useCallback(() => {
    setForm(defaultForm());
    addToast("已清空", "info");
  }, [addToast]);

  const handleUpdateClick = useCallback(async () => {
    if (updating) return;
    setUpdating(true);
    setUpdateProgress(0);
    setUpdateMessage("正在检查更新...");
    try {
      const result = await api.checkUpdate(false, false);
      setUpdateCheckResult(result);
      if (!result.has_data_update && !result.has_app_update) {
        setUpdateMessage("已是最新版本");
        addToast("已是最新版本", "info");
        setUpdating(false);
        return;
      }
      setUpdateMessage(`发现 ${result.data_files_to_update.length} 个文件需要更新`);
      addToast("发现更新", "info");

      const unlisten = api.listenUpdateProgress((evt: UpdateProgressEvent) => {
        setUpdateProgress(evt.progress);
        setUpdateMessage(`正在下载: ${evt.file || evt.message}`);
      });

      await api.performUpdate(false, result);
      unlisten();
      setUpdateProgress(1);
      setUpdateMessage("更新完成");
      addToast("更新完成", "success");
    } catch (err) {
      setUpdateMessage("更新失败");
      addToast(String(err), "error");
    } finally {
      setUpdating(false);
    }
  }, [updating, addToast]);

  // Load initial profession config on mount
  useEffect(() => {
    handleXinfaChange(form.xinfa);
  }, []);

  return (
    <div className={styles.card}>
      <div className={styles.section}>
        <div className={styles.sectionTitle}>心法</div>
        <select
          className={styles.select}
          value={form.xinfa}
          onChange={(e) => handleXinfaChange(e.target.value)}
        >
          {XINFA_LIST.map((x) => (
            <option key={x.value} value={x.value}>{x.label}</option>
          ))}
        </select>
      </div>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>玩家属性</div>
        <div className={styles.grid}>
          {PLAYER_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input
                className={styles.input}
                type="number"
                min={f.min}
                step={f.step}
                value={form.player[f.id] ?? ""}
                onChange={(e) => updateField("player", f.id, e.target.value)}
              />
            </div>
          ))}
        </div>
      </div>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>目标属性</div>
        <div className={styles.grid}>
          {HOSTILE_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input
                className={styles.input}
                type="number"
                min={f.min}
                max={f.max}
                step={f.step}
                value={form.hostile[f.id] ?? ""}
                onChange={(e) => updateField("hostile", f.id, e.target.value)}
              />
            </div>
          ))}
        </div>
      </div>

      <div className={styles.section}>
        <div className={styles.actions}>
          <button
            className={`${styles.btn} ${styles.btnPrimary}`}
            onClick={handleCalculate}
            disabled={calculating}
          >
            {calculating ? "计算中..." : "开始计算"}
          </button>
          <button className={styles.btn} onClick={handleSave}>保存</button>
          <button className={styles.btn} onClick={handleLoad}>加载</button>
          <button className={`${styles.btn} ${styles.btnDanger}`} onClick={handleClear}>
            清空
          </button>
          <button
            className={styles.btn}
            onClick={handleUpdateClick}
            disabled={updating}
          >
            {updating ? "更新中..." : "检查更新"}
          </button>
        </div>
        {updating && (
          <div className={styles.progress}>
            <div className={styles.progressTrack}>
              <div
                className={styles.progressFill}
                style={{ width: `${Math.round(updateProgress * 100)}%` }}
              />
            </div>
            <div className={styles.progressText}>{updateMessage}</div>
          </div>
        )}
      </div>
    </div>
  );
}
