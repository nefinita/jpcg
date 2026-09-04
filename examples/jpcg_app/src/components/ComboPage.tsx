import { useState, useEffect, useCallback, useMemo } from "react";
import { DragDropContext, Droppable, Draggable } from "@hello-pangea/dnd";
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import type {
  SkillPoolItemDTO, ComboStepDTO, StepOverrideDTO, ComboResultDTO, FormData,
} from "../types";
import * as api from "../api/commands";
import { toCalculateRequest } from "../utils/normalize";
import { IconGear, IconClose, IconSave, IconTrash, IconStar } from "./icons";
import styles from "./ComboPage.module.css";

interface Props {
  xinfaName?: string;
  formData?: FormData | null;
}

export default function ComboPage({ xinfaName, formData }: Props) {
  const [skillPool, setSkillPool] = useState<SkillPoolItemDTO[]>([]);
  const [sequence, setSequence] = useState<ComboStepDTO[]>([]);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const [presets, setPresets] = useState<string[]>([]);
  const [comboResult, setComboResult] = useState<ComboResultDTO | null>(null);
  const [computing, setComputing] = useState(false);
  const [adjustTarget, setAdjustTarget] = useState<number | null>(null);
  const [saveDialog, setSaveDialog] = useState(false);
  const [presetName, setPresetName] = useState("");
  const [poolQuery, setPoolQuery] = useState("");
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (xinfaName) {
      api.loadSkillPool(xinfaName).then(setSkillPool).catch(() => {});
    }
    api.listComboPresets().then(setPresets).catch(() => {});
    const stored = localStorage.getItem("jpcg_favorites");
    if (stored) {
      try {
        const raw = JSON.parse(stored);
        // 兼容旧格式（纯 skill_id 数组）与新格式（"skill_id-sub_id" 字符串）
        const favs = new Set<string>((Array.isArray(raw) ? raw : []).map((v: unknown) =>
          typeof v === "number" ? String(v) : String(v),
        ));
        setFavorites(favs);
      } catch {}
    }
  }, [xinfaName]);

  const saveFavorites = useCallback((favs: Set<string>) => {
    setFavorites(favs);
    localStorage.setItem("jpcg_favorites", JSON.stringify([...favs]));
  }, []);

  const pool = skillPool;

  // 技能唯一标识（同 skill_id 不同 sub_id 形态区分）
  const skillKey = useCallback((s: SkillPoolItemDTO) => `${s.skill_id}-${s.sub_id ?? 0}`, []);

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

  const toggleFavorite = useCallback((s: SkillPoolItemDTO) => {
    const key = `${s.skill_id}-${s.sub_id ?? 0}`;
    saveFavorites(new Set(favorites.has(key) ? [...favorites].filter((k) => k !== key) : [...favorites, key]));
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
    if (sequence.length === 0 || !formData) return;
    setComputing(true);
    try {
      const data = formData;
      const req = toCalculateRequest(data);
      const result = await api.calculateCombo(
        sequence,
        req.player,
        req.hostile,
        req.xinfa_config,
        req.buff,
        req.coefficient,
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

  // ===== 技能池：搜索 + 按基础名分组折叠 =====
  const baseNameOf = useCallback((s: SkillPoolItemDTO) => {
    // 基础名 = 去掉「·形态后缀」（如 引窍·0点任脉 → 引窍）或 (lvN)/（dot）等后缀
    let n = s.skill_name;
    const dot = n.indexOf("（dot）");
    if (dot !== -1) n = n.slice(0, dot) + "·dot";
    const lv = n.search(/[\(（]lv\d+[\)）]$/);
    if (lv !== -1) n = n.slice(0, lv);
    const sep = n.lastIndexOf("·");
    if (sep !== -1) n = n.slice(0, sep);
    return n || s.skill_name;
  }, []);

  const filteredPool = useMemo(() => {
    const q = poolQuery.trim().toLowerCase();
    if (!q) return pool;
    return pool.filter((s) => s.skill_name.toLowerCase().includes(q));
  }, [pool, poolQuery]);

  const groups = useMemo(() => {
    const m = new Map<string, SkillPoolItemDTO[]>();
    for (const s of filteredPool) {
      const key = baseNameOf(s);
      const arr = m.get(key) || [];
      arr.push(s);
      m.set(key, arr);
    }
    return [...m.entries()].sort((a, b) => a[0].localeCompare(b[0], "zh-CN"));
  }, [filteredPool, baseNameOf]);

  const toggleGroup = useCallback((name: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name); else next.add(name);
      return next;
    });
  }, []);

  const expandAll = useCallback(() => setCollapsedGroups(new Set()), []);
  const collapseAll = useCallback(() => {
    setCollapsedGroups(new Set(groups.filter(([, items]) => items.length > 1).map(([name]) => name)));
  }, [groups]);

  // 需先同步面板属性（未计算过时 formData 为 null，全 0 提交会让追加真伤恒为 0）
  const noForm = !formData;
  const targetHp = Number(formData?.hostile?.target_hp) || 0;
  const hasLostHpSkills = sequence.some((s) => (s.skill.lost_hp_zhenshishanghai ?? 0) > 0);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2>排轴器</h2>
        <div className={styles.headerActions}>
          {sequence.length > 0 && (
            <>
              <button className={styles.primaryBtn} onClick={handleCalculate} disabled={computing || noForm}
                title={noForm ? "请先在「计算器」页输入属性并点击计算，同步面板与目标血量" : ""}>
                {computing ? "计算中..." : "计算连招"}
              </button>
              <button className={styles.btn} onClick={clearSequence}>清空</button>
            </>
          )}
        </div>
      </div>

      {noForm && (
        <div className={styles.warnBanner}>
          尚未同步面板属性：请先在「计算器」页输入属性并点击计算，再回来排轴（否则以 0 属性提交，追加真伤不会生效）。
        </div>
      )}
      {!noForm && targetHp <= 0 && (
        <div className={styles.warnBanner}>
          目标血量 = 0：追加真伤（已损失生命值 × 系数）不会生效，请在计算器页设置目标血量。
        </div>
      )}
      {hasLostHpSkills && !noForm && targetHp > 0 && (
        <div className={styles.zhenshiHint}>
          连招含追加真伤技能（已损失生命值 × 系数，无视防御）：首击满血追加为 0，随血量损耗递增，见逐步骤「追加真伤」列。
        </div>
      )}

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
                      <Draggable key={`${item.skill.skill_id}-${item.skill.sub_id ?? 0}-${i}`} draggableId={`seq-${i}`} index={i}>
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
        <div className={styles.poolToolbar}>
          <input
            className={styles.poolSearch}
            type="search"
            placeholder="搜索技能（如 引窍 / 0点任脉）..."
            value={poolQuery}
            onChange={(e) => setPoolQuery(e.target.value)}
          />
          <button className={styles.smallBtn} onClick={expandAll} title="展开全部组">展开</button>
          <button className={styles.smallBtn} onClick={collapseAll} title="折叠全部组">折叠</button>
        </div>
        <div className={styles.comboPool}>
          {groups.map(([name, items]) => {
            const collapsed = collapsedGroups.has(name);
            const hasMulti = items.length > 1;
            return (
              <div key={name} className={styles.poolGroup}>
                <button
                  className={styles.poolGroupHeader}
                  onClick={() => hasMulti && toggleGroup(name)}
                  title={hasMulti ? "点击折叠/展开" : ""}
                >
                  <span className={styles.poolGroupArrow}>{hasMulti ? (collapsed ? "▸" : "▾") : ""}</span>
                  <span className={styles.poolGroupName}>{name}</span>
                  {hasMulti && <span className={styles.poolGroupCount}>{items.length}种形态</span>}
                </button>
                {!collapsed && (
                  <div className={styles.poolGroupItems}>
                    {items.map((s) => {
                      const key = skillKey(s);
                      const fav = favorites.has(key);
                      return (
                        <button key={key}
                          className={`${styles.skillChip} ${fav ? styles.favoriteChip : ""}`}
                          onClick={() => addToSequence(s)}
                          onContextMenu={(e) => { e.preventDefault(); toggleFavorite(s); }}
                          title={`${s.skill_name} | 基础伤害 ${s.base_damage1}-${s.base_damage2} | 系数 ${s.atk_xishu}${s.lost_hp_zhenshishanghai > 0 ? ` | 追加真伤 ${(s.lost_hp_zhenshishanghai * 100).toFixed(0)}%已损失` : ""}${fav ? " · 已收藏" : " · 右键标记收藏"}`}>
                          {s.skill_name}
                          {s.lost_hp_zhenshishanghai > 0 && (
                            <span className={styles.comboZhenshiTag}>真伤{(s.lost_hp_zhenshishanghai * 100).toFixed(0)}%</span>
                          )}
                          {fav && <span className={styles.starIcon}><IconStar size={12} /></span>}
                        </button>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
          {groups.length === 0 && (
            <span className={styles.emptyHint}>{pool.length === 0 ? "先选择心法加载技能池" : "没有匹配的技能"}</span>
          )}
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
    真伤: Math.round((s.lost_hp_zhenshi_damage ?? 0) / 10000 * 10) / 10,
  }));

  const zhenshiTotal = result.steps.reduce((sum, s) => sum + (s.lost_hp_zhenshi_damage ?? 0), 0);
  const hasZhenshi = zhenshiTotal > 0;

  return (
    <section className={styles.comboResult}>
      <div className={styles.comboResultSummary}>
        <span>总期望: <strong>{result.total_expected_damage_wan.toFixed(1)}万</strong></span>
        {hasZhenshi && <span>追加真伤: <strong>{(zhenshiTotal / 10000).toFixed(1)}万</strong></span>}
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
              {hasZhenshi && <Bar dataKey="真伤" fill="#ef8354" stackId="zhenshi" radius={[3, 3, 0, 0]} />}
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
              {hasZhenshi && <th>追加真伤</th>}
              <th>累计(万)</th><th>击杀概率</th>
            </tr>
          </thead>
          <tbody>
            {result.steps.map((s, i) => {
              const isFixed = s.has_critical_strike || (s.zhenshishanghai ?? 0) > 0;
              return (
              <tr key={i}>
                <td>
                  {s.skill_name}
                  {s.has_critical_strike && (
                    <span className={styles.comboWuzhiTag}>无质</span>
                  )}
                  {(s.zhenshishanghai ?? 0) > 0 && (
                    <span className={styles.comboWuzhiTag}>真实</span>
                  )}
                  {(s.lost_hp_zhenshi_damage ?? 0) > 0 && (
                    <span className={styles.comboZhenshiTag}>追加真伤</span>
                  )}
                  {s.dot_jumps?.length > 0 && (
                    <div className={styles.comboDotJumps}>
                      {s.dot_jumps.map((j, k) => (
                        <span key={k} title={`第${k + 1}跳`}>
                          {k + 1}:{Math.round(j / 10000 * 10) / 10}万
                        </span>
                      ))}
                    </div>
                  )}
                </td>
                {isFixed ? (
                  <>
                    <td>-</td>
                    <td>-</td>
                  </>
                ) : (
                  <>
                    <td>{Math.round(s.g_damage / 10000 * 10) / 10}万</td>
                    <td>{Math.round(s.h_damage / 10000 * 10) / 10}万</td>
                  </>
                )}
                <td>{Math.round(s.q_damage / 10000 * 10) / 10}万</td>
                {hasZhenshi && (
                  <td>
                    {(s.lost_hp_zhenshi_damage ?? 0) > 0
                      ? `${Math.round(s.lost_hp_zhenshi_damage / 10000 * 10) / 10}万`
                      : "-"}
                  </td>
                )}
                <td>{s.cumulative_mean_wan.toFixed(1)}</td>
                <td>{(s.kill_prob * 100).toFixed(1)}%</td>
              </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
