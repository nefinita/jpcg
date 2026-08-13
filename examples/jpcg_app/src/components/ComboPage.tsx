import { useState, useEffect, useCallback } from "react";
import { DragDropContext, Droppable, Draggable } from "@hello-pangea/dnd";
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import type {
  SkillPoolItemDTO, ComboStepDTO, StepOverrideDTO, ComboResultDTO, FormData,
} from "../types";
import * as api from "../api/commands";
import { IconGear, IconClose, IconSave, IconTrash, IconStar } from "./icons";
import styles from "./ComboPage.module.css";

interface Props {
  xinfaName?: string;
  formData?: FormData | null;
}

export default function ComboPage({ xinfaName, formData }: Props) {
  const [skillPool, setSkillPool] = useState<SkillPoolItemDTO[]>([]);
  const [sequence, setSequence] = useState<ComboStepDTO[]>([]);
  const [favorites, setFavorites] = useState<Set<number>>(new Set());
  const [presets, setPresets] = useState<string[]>([]);
  const [comboResult, setComboResult] = useState<ComboResultDTO | null>(null);
  const [computing, setComputing] = useState(false);
  const [adjustTarget, setAdjustTarget] = useState<number | null>(null);
  const [saveDialog, setSaveDialog] = useState(false);
  const [presetName, setPresetName] = useState("");

  useEffect(() => {
    if (xinfaName) {
      api.loadSkillPool(xinfaName).then(setSkillPool).catch(() => {});
    }
    api.listComboPresets().then(setPresets).catch(() => {});
    const stored = localStorage.getItem("jpcg_favorites");
    if (stored) {
      try { setFavorites(new Set(JSON.parse(stored))); } catch {}
    }
  }, [xinfaName]);

  const saveFavorites = useCallback((favs: Set<number>) => {
    setFavorites(favs);
    localStorage.setItem("jpcg_favorites", JSON.stringify([...favs]));
  }, []);

  const pool = skillPool;

  const addToSequence = useCallback((skill: SkillPoolItemDTO) => {
    setSequence((prev) => [...prev, { skill, overrides: null }]);
  }, []);

  const removeFromSequence = useCallback((index: number) => {
    setSequence((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearSequence = useCallback(() => {
    setSequence([]);
    setComboResult(null);
  }, []);

  const toggleFavorite = useCallback((skillId: number) => {
    saveFavorites(new Set(favorites.has(skillId) ? [...favorites].filter((id) => id !== skillId) : [...favorites, skillId]));
  }, [favorites, saveFavorites]);

  const onDragEnd = useCallback((result: any) => {
    if (!result.destination) return;
    const items = Array.from(sequence);
    const [removed] = items.splice(result.source.index, 1);
    items.splice(result.destination.index, 0, removed);
    setSequence(items);
  }, [sequence]);

  const updateOverride = useCallback((index: number, overrides: StepOverrideDTO) => {
    setSequence((prev) => prev.map((s, i) => i === index ? { ...s, overrides } : s));
  }, []);

  const resetOverride = useCallback((index: number) => {
    setSequence((prev) => prev.map((s, i) => i === index ? { ...s, overrides: null } : s));
  }, []);

  const handleCalculate = useCallback(async () => {
    if (sequence.length === 0) return;
    setComputing(true);
    try {
      const data = formData || { player: {}, hostile: {}, xinfa_config: {} } as FormData;
      const result = await api.calculateCombo(
        sequence,
        data.player || {},
        data.hostile || {},
        data.xinfa_config || {},
        data.buff || { base_atk_pct: 0, huixin_pct: 0, huixiao_pct: 0, pofang_pct: 0, wushi_fangyu_pct: 0, shanghai_pct: 0, mode_is_point: false },
        data.coefficient || { pofang_xishu: 225957.6, huixin_xishu: 197703, huixiao_xishu: 72844.2, huajin_xishu: 30115.8, fangyu_xishu: 126007.2, pvp_global_jianshang: 0.9 },
      );
      setComboResult(result);
    } catch (err) {
      console.error(err);
    } finally {
      setComputing(false);
    }
  }, [sequence, formData]);

  const handleSavePreset = useCallback(async () => {
    if (!presetName.trim()) return;
    try {
      await api.saveComboPreset(presetName.trim(), sequence);
      setPresetName("");
      setSaveDialog(false);
      setPresets(await api.listComboPresets());
    } catch (err) {
      console.error(err);
    }
  }, [presetName, sequence]);

  const handleLoadPreset = useCallback(async (name: string) => {
    try {
      const preset = await api.loadComboPreset(name);
      setSequence(preset.steps);
    } catch (err) {
      console.error(err);
    }
  }, []);

  const handleDeletePreset = useCallback(async (name: string) => {
    try {
      await api.deleteComboPreset(name);
      setPresets(await api.listComboPresets());
    } catch (err) {
      console.error(err);
    }
  }, []);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2>排轴器</h2>
        <div className={styles.headerActions}>
          {sequence.length > 0 && (
            <>
              <button className={styles.primaryBtn} onClick={handleCalculate} disabled={computing}>
                {computing ? "计算中..." : "计算连招"}
              </button>
              <button className={styles.btn} onClick={clearSequence}>清空</button>
            </>
          )}
        </div>
      </div>

      {/* Sequence */}
      <section className={styles.section}>
        <div className={styles.sectionTitle}>连招序列</div>
        <div className={styles.comboSequence}>
          {sequence.length === 0 ? (
            <span className={styles.emptyHint}>从下方技能池添加技能到序列</span>
          ) : (
            <DragDropContext onDragEnd={onDragEnd}>
              <Droppable droppableId="sequence" direction="horizontal">
                {(provided) => (
                  <div className={styles.sequenceRow} ref={provided.innerRef} {...provided.droppableProps}>
                    {sequence.map((item, i) => (
                      <Draggable key={`${item.skill.skill_id}-${i}`} draggableId={`seq-${i}`} index={i}>
                        {(provided, snapshot) => (
                          <div ref={provided.innerRef} {...provided.draggableProps} {...provided.dragHandleProps}
                            className={`${styles.comboItem} ${snapshot.isDragging ? styles.dragging : ""}`}>
                            <span className={styles.comboIndex}>{i + 1}</span>
                            <span className={styles.comboName}>{item.skill.skill_name}</span>
                            <span className={styles.comboActions}>
                              <button className={styles.smallBtn} onClick={() => setAdjustTarget(adjustTarget === i ? null : i)} title="调整系数"><IconGear size={14} /></button>
                              <button className={styles.smallBtn} onClick={() => removeFromSequence(i)} title="移除"><IconClose size={14} /></button>
                            </span>
                          </div>
                        )}
                      </Draggable>
                    ))}
                    {provided.placeholder}
                  </div>
                )}
              </Droppable>
            </DragDropContext>
          )}
        </div>
      </section>

      {/* Step adjust modal */}
      {adjustTarget !== null && (
        <StepAdjustModal
          step={sequence[adjustTarget]}
          index={adjustTarget}
          onUpdate={updateOverride}
          onReset={resetOverride}
          onClose={() => setAdjustTarget(null)}
        />
      )}

      {/* Combo result */}
      {comboResult && (
        <ComboResultDisplay result={comboResult} />
      )}

      {/* Preset management */}
      <section className={styles.section}>
        <div className={styles.presetBar}>
          <span className={styles.sectionTitle}>预设</span>
          <select className={styles.presetSelect} defaultValue="" onChange={(e) => e.target.value && handleLoadPreset(e.target.value)}>
            <option value="" disabled>加载预设...</option>
            {presets.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
          <button className={styles.smallBtn} onClick={() => setSaveDialog(true)}><IconSave size={14} /> 保存</button>
          {presets.length > 0 && (
            <button className={styles.smallBtn} onClick={() => { const p = presets[presets.length - 1]; handleDeletePreset(p); }}><IconTrash size={14} /></button>
          )}
        </div>

        {saveDialog && (
          <div className={styles.saveDialog}>
            <input className={styles.input} placeholder="连招名称" value={presetName} onChange={(e) => setPresetName(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleSavePreset(); if (e.key === "Escape") setSaveDialog(false); }} autoFocus />
            <button className={styles.btn} onClick={handleSavePreset}>确认保存</button>
            <button className={styles.btn} onClick={() => setSaveDialog(false)}>取消</button>
          </div>
        )}
      </section>

      {/* Skill pool */}
      <section className={styles.section}>
        <div className={styles.sectionTitle}>
          技能池
          <span className={styles.poolCount}>{pool.length}个</span>
        </div>
        <div className={styles.comboPool}>
          {pool.map((s, i) => (
            <button key={s.skill_id || i}
              className={`${styles.skillChip} ${favorites.has(s.skill_id) ? styles.favoriteChip : ""}`}
              onClick={() => addToSequence(s)}
              onContextMenu={(e) => { e.preventDefault(); toggleFavorite(s.skill_id); }}
              title={`基础伤害 ${s.base_damage1}-${s.base_damage2} | 系数 ${s.atk_xishu}${favorites.has(s.skill_id) ? " · 已收藏" : " · 右键标记收藏"}`}>
              {s.skill_name}
              {favorites.has(s.skill_id) && <span className={styles.starIcon}><IconStar size={12} /></span>}
            </button>
          ))}
          {pool.length === 0 && <span className={styles.emptyHint}>先选择心法加载技能池</span>}
        </div>
      </section>
    </div>
  );
}

function StepAdjustModal({ step, index, onUpdate, onReset, onClose }: {
  step: ComboStepDTO; index: number;
  onUpdate: (i: number, o: StepOverrideDTO) => void;
  onReset: (i: number) => void;
  onClose: () => void;
}) {
  const [overrides, setOverrides] = useState<StepOverrideDTO>(
    step.overrides || {
      base_damage_override: null, atk_xishu_override: null, jianshang_bili_override: null,
      wushihuajin_override: null, extra_atk_pct: null, gain_override: null,
      extra_crit_pct: null, extra_crit_dmg_pct: null,
    },
  );

  const fields = [
    { key: "base_damage_override" as const, label: "基础伤害", original: `${(step.skill.base_damage1 + step.skill.base_damage2) / 2}` },
    { key: "atk_xishu_override" as const, label: "技能系数", original: String(step.skill.atk_xishu) },
    { key: "jianshang_bili_override" as const, label: "减伤(%)", original: "0" },
    { key: "wushihuajin_override" as const, label: "无视化劲(%)", original: String(step.skill.wushihuajin) },
    { key: "extra_atk_pct" as const, label: "额外攻击(%)", original: "0" },
    { key: "gain_override" as const, label: "郭氏增益(%)", original: "0" },
    { key: "extra_crit_pct" as const, label: "额外会心(%)", original: String(step.skill.huixin_up) },
    { key: "extra_crit_dmg_pct" as const, label: "额外会效(%)", original: String(step.skill.huixiao_up) },
  ];

  const handleChange = (key: keyof StepOverrideDTO, value: string) => {
    const num = value === "" ? null : Number(value);
    setOverrides((prev) => ({ ...prev, [key]: isNaN(num as number) ? null : num }));
  };

  const handleSave = () => {
    onUpdate(index, overrides);
    onClose();
  };

  const handleReset = () => {
    const empty = {
      base_damage_override: null, atk_xishu_override: null, jianshang_bili_override: null,
      wushihuajin_override: null, extra_atk_pct: null, gain_override: null,
      extra_crit_pct: null, extra_crit_dmg_pct: null,
    };
    setOverrides(empty);
    onReset(index);
    onClose();
  };

  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.modalHeader}>
          <span>{step.skill.skill_name} — 临时调整</span>
          <button onClick={onClose} aria-label="关闭"><IconClose size={15} /></button>
        </div>
        <div className={styles.modalBody}>
          {fields.map((f) => (
            <div key={f.key} className={styles.modalField}>
              <label>{f.label}</label>
              <input type="number" step={0.1}
                value={overrides[f.key] ?? ""}
                placeholder={`原始: ${f.original}`}
                onChange={(e) => handleChange(f.key, e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleSave(); if (e.key === "Escape") onClose(); }} />
            </div>
          ))}
        </div>
        <div className={styles.modalActions}>
          <button className={styles.btn} onClick={handleSave}>保存调整</button>
          <button className={styles.btn} onClick={handleReset}>恢复原始</button>
        </div>
      </div>
    </div>
  );
}

function ComboResultDisplay({ result }: { result: ComboResultDTO }) {

  const curveData = result.kill_prob_curve.map(([step, prob]) => ({
    step: `第${step}步`,
    prob: Math.round(prob * 100) / 100,
  }));

  const barData = result.steps.map((s) => ({
    name: s.skill_name.length > 4 ? s.skill_name.slice(0, 4) + ".." : s.skill_name,
    期望: Math.round(s.q_damage / 10000 * 10) / 10,
    会心: Math.round(s.h_damage / 10000 * 10) / 10,
  }));

  return (
    <section className={styles.comboResult}>
      <div className={styles.comboResultSummary}>
        <span>总期望: <strong>{result.total_expected_damage_wan.toFixed(1)}万</strong></span>
        <span>击杀概率: <strong>{(result.final_kill_prob * 100).toFixed(1)}%</strong></span>
        <span>技能数: {result.steps.length}</span>
      </div>

      <div className={styles.comboChartSection}>
        <div className={styles.sectionLabel}>击杀概率曲线</div>
        <div className={styles.chartContainer}>
          <ResponsiveContainer width="100%" height={180}>
            <LineChart data={curveData}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--border-subtle)" />
              <XAxis dataKey="step" tick={{ fontSize: 10 }} />
              <YAxis domain={[0, 100]} tick={{ fontSize: 10 }} tickFormatter={(v: any) => Number(v) + "%"} />
              <Tooltip formatter={(v: any) => Number(v).toFixed(1) + "%"} />
              <Line type="monotone" dataKey="prob" stroke="var(--primary-500)" strokeWidth={2} dot={{ r: 3 }} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className={styles.comboChartSection}>
        <div className={styles.sectionLabel}>每步伤害对比</div>
        <div className={styles.chartContainer}>
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={barData}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--border-subtle)" />
              <XAxis dataKey="name" tick={{ fontSize: 9 }} />
              <YAxis tick={{ fontSize: 10 }} tickFormatter={(v: any) => Number(v) + "万"} />
              <Tooltip formatter={(v: any) => Number(v).toFixed(1) + "万"} />
              <Bar dataKey="期望" fill="var(--primary-500)" radius={[3, 3, 0, 0]} />
              <Bar dataKey="会心" fill="var(--accent-500)" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className={styles.comboChartSection}>
        <div className={styles.sectionLabel}>逐步骤详细数据</div>
        <table className={styles.comboDetailTable}>
          <thead>
            <tr>
              <th>技能</th><th>普伤</th><th>会心</th><th>期望</th>
              <th>累计(万)</th><th>击杀概率</th>
            </tr>
          </thead>
          <tbody>
            {result.steps.map((s, i) => (
              <tr key={i}>
                <td>
                  {s.skill_name}
                  {s.dot_jumps?.length > 0 && (
                    <div className={styles.comboDotJumps}>
                      {s.dot_jumps.map((j, k) => (
                        <span key={k} title={`第${k + 1}跳`}>
                          {Math.round(j / 10000 * 10) / 10}万
                        </span>
                      ))}
                    </div>
                  )}
                </td>
                <td>{Math.round(s.g_damage / 10000 * 10) / 10}万</td>
                <td>{Math.round(s.h_damage / 10000 * 10) / 10}万</td>
                <td>{Math.round(s.q_damage / 10000 * 10) / 10}万</td>
                <td>{s.cumulative_mean_wan.toFixed(1)}</td>
                <td>{(s.kill_prob * 100).toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
