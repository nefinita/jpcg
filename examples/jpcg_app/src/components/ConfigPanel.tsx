import React, { useState, useCallback, useEffect, useRef } from "react";
import type { FormData, UpdateProgressEvent, UpdateCheckResult, BuffConfigDTO } from "../types";
import {
  XINFA_LIST as XINFA_FALLBACK, PLAYER_FIELDS, HOSTILE_FIELDS, STORAGE_KEYS,
  BUFF_FIELDS, COEFFICIENT_FIELDS, DEFAULT_BUFF, DEFAULT_COEFFICIENT,
} from "../utils/constants";
import * as api from "../api/commands";
import styles from "./ConfigPanel.module.css";

interface Props {
  onCalculate: (form: FormData) => void;
  calculating: boolean;
  addToast: (msg: string, type?: "success" | "error" | "warning" | "info") => void;
  setStatus: (msg: string) => void;
  onXinfaChange?: (xinfa: string) => void;
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
  buff: { ...DEFAULT_BUFF },
  coefficient: { ...DEFAULT_COEFFICIENT },
});

export default function ConfigPanel({ onCalculate, calculating, addToast, setStatus, onXinfaChange }: Props) {
  const [form, setForm] = useState<FormData>(() => {
    const last = typeof localStorage !== "undefined"
      ? localStorage.getItem(STORAGE_KEYS.lastXinfa)
      : null;
    const defaultXinfa = last || "mowen";
    const entry = XINFA_FALLBACK.find((x) => x.value === defaultXinfa) || XINFA_FALLBACK[0];
    return { ...defaultForm(), xinfa: defaultXinfa, xinfa_config: { ...defaultForm().xinfa_config, xinfa_name: entry.label } };
  });

  const [professionOptions, setProfessionOptions] = useState<{value: string; label: string; nom: string; version_label: string | null}[]>(
    () => XINFA_FALLBACK.map((x) => ({value: x.value, label: x.label, nom: "", version_label: null}))
  );
  const [updating, setUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [updateMessage, setUpdateMessage] = useState("");
  const [updateCheckResult, setUpdateCheckResult] = useState<UpdateCheckResult | null>(null);

  useEffect(() => {
    api.listProfessions().then((list) => {
      if (list.length > 0) setProfessionOptions(list);
    }).catch(() => {});
  }, []);

  const defaultXinfa = XINFA_FALLBACK.find((x) => x.default) || XINFA_FALLBACK[0];

  const handleXinfaChange = useCallback((value: string) => {
    const entry = professionOptions.find((x) => x.value === value);
    localStorage.setItem(STORAGE_KEYS.lastXinfa, value);
    setForm((prev): FormData => ({
      ...prev,
      xinfa: value,
      xinfa_config: { ...prev.xinfa_config, xinfa_name: entry?.label ?? defaultXinfa.label },
    }));
    onXinfaChange?.(value);
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
  }, [professionOptions, defaultXinfa, onXinfaChange]);

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

  const updateBuff = useCallback((id: string, value: string) => {
    const num = value === "" ? 0 : Number(value);
    setForm((prev) => ({
      ...prev,
      buff: { ...prev.buff, [id]: isNaN(num) ? 0 : num },
    }));
  }, []);

  const updateCoefficient = useCallback((id: string, value: string) => {
    const num = value === "" ? 0 : Number(value);
    setForm((prev) => ({
      ...prev,
      coefficient: { ...prev.coefficient, [id]: isNaN(num) ? 0 : num },
    }));
  }, []);

  const handleCalculate = useCallback(() => {
    onCalculate(form);
  }, [form, onCalculate]);

  const handleSave = useCallback(async () => {
    try {
      await api.saveConfig({
        player: form.player as never,
        hostile: form.hostile as never,
        xinfa_config: form.xinfa_config,
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
      const entry = professionOptions.find((x) => x.label === cfg.xinfa_config.xinfa_name);
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
          target_hp: cfg.hostile.target_hp,
        },
        xinfa_config: cfg.xinfa_config,
        buff: cfg.buff || { ...DEFAULT_BUFF },
        coefficient: cfg.coefficient || { ...DEFAULT_COEFFICIENT },
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

  const handleExport = useCallback(async () => {
    try {
      const toml = await api.exportConfig();
      const blob = new Blob([toml], { type: "application/toml" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${form.xinfa}_config.toml`;
      a.click();
      URL.revokeObjectURL(url);
      addToast("配置已导出", "success");
    } catch (err) {
      addToast(String(err), "error");
    }
  }, [form, addToast]);

  const handleImport = useCallback(async () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".toml";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        await api.importConfig(text);
        addToast("配置已导入", "success");
        handleLoad();
      } catch (err) {
        addToast(String(err), "error");
      }
    };
    input.click();
  }, [addToast, handleLoad]);

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

  useEffect(() => {
    handleXinfaChange(form.xinfa);
  }, []);

  const playerStats = React.useMemo(() => {
    const pct = (v: number) => (v * 100).toFixed(1) + "%";
    const huixinRate = form.player.huixin_dengji / (form.coefficient.huixin_xishu || 197703);
    const pofangRate = form.player.pofang_dengji / (form.coefficient.pofang_xishu || 225957.6);
    return {
      huixinRate: pct(huixinRate + form.buff.huixin_pct / 100),
      pofangRate: pct(pofangRate + form.buff.pofang_pct / 100),
      jianshang: form.hostile.jianshang_bili + "%",
    };
  }, [form]);

  return (
    <div className={styles.card}>
      <div className={styles.section}>
        <div className={styles.sectionTitle}>心法</div>
        <select
          className={styles.select}
          value={form.xinfa}
          onChange={(e) => handleXinfaChange(e.target.value)}
        >
          {professionOptions.map((x) => (
            <option key={x.value} value={x.value}>{x.label}（{x.nom}）{x.version_label ?? ""}</option>
          ))}
        </select>
      </div>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>玩家属性</div>
        <div className={styles.grid}>
          {PLAYER_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input className={styles.input} type="number" min={f.min} step={f.step}
                value={form.player[f.id] ?? ""}
                onChange={(e) => updateField("player", f.id, e.target.value)} />
            </div>
          ))}
        </div>
      </div>

      <details className={styles.details}>
        <summary className={styles.sectionTitle}>阵眼/奇穴增益</summary>
        <div className={styles.grid}>
          {BUFF_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input className={styles.input} type="number" min={0} step={0.1}
                value={String(form.buff[f.id as keyof BuffConfigDTO] ?? "")}
                onChange={(e) => updateBuff(f.id, e.target.value)} />
            </div>
          ))}
        </div>
      </details>

      <details className={styles.details}>
        <summary className={styles.sectionTitle}>系数设置</summary>
        <div className={styles.grid}>
          {COEFFICIENT_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input className={styles.input} type="number" min={0} step={0.1}
                value={form.coefficient[f.id as keyof typeof form.coefficient] ?? ""}
                onChange={(e) => updateCoefficient(f.id, e.target.value)} />
            </div>
          ))}
        </div>
      </details>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>目标属性</div>
        <div className={styles.grid}>
          {HOSTILE_FIELDS.map((f) => (
            <div key={f.id} className={styles.field}>
              <label className={styles.fieldLabel}>{f.label}</label>
              <input className={styles.input} type="number" min={f.min} max={f.max} step={f.step}
                value={form.hostile[f.id] ?? ""}
                onChange={(e) => updateField("hostile", f.id, e.target.value)} />
            </div>
          ))}
        </div>
      </div>

      <div className={styles.statBar}>
        <span>会心率: {playerStats.huixinRate}</span>
        <span>破防率: {playerStats.pofangRate}</span>
        <span>减伤: {playerStats.jianshang}</span>
      </div>

      <div className={styles.section}>
        <div className={styles.actions}>
          <button className={`${styles.btn} ${styles.btnPrimary}`}
            onClick={handleCalculate} disabled={calculating}>
            {calculating ? "计算中..." : "开始计算"}
          </button>
          <button className={styles.btn} onClick={handleSave}>保存</button>
          <button className={styles.btn} onClick={handleLoad}>加载</button>
          <button className={`${styles.btn} ${styles.btnDanger}`} onClick={handleClear}>清空</button>
          <button className={styles.btn} onClick={handleExport}>导出</button>
          <button className={styles.btn} onClick={handleImport}>导入</button>
          <button className={styles.btn} onClick={handleUpdateClick} disabled={updating}>
            {updating ? "更新中..." : "检查更新"}
          </button>
        </div>
        {updating && (
          <div className={styles.progress}>
            <div className={styles.progressTrack}>
              <div className={styles.progressFill} style={{ width: `${Math.round(updateProgress * 100)}%` }} />
            </div>
            <div className={styles.progressText}>{updateMessage}</div>
          </div>
        )}
      </div>
    </div>
  );
}
