import { useState, useEffect, useCallback, useMemo, useRef, type SetStateAction, type Dispatch } from "react";
import { DragDropContext, Droppable, Draggable } from "@hello-pangea/dnd";
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import type {
  SkillPoolItemDTO, ComboStepDTO, StepOverrideDTO, ComboResultDTO, FormData, TimelineResultDTO, CombatBundleDTO, CombatDataDTO,
} from "../types";
import * as api from "../api/commands";
import { toCalculateRequest } from "../utils/normalize";
import { IconGear, IconClose, IconSave, IconTrash, IconStar } from "./icons";
import { usePersistentState, readPreference, writePreference } from "../hooks/usePersistentState";
import { STORAGE_KEYS } from "../utils/constants";
import { comboDraftSchema, defaultComboDraft, type ComboDraft } from "../state/cockpit";
import { formatTimelineEvent } from "./timelineFormat";
import { TimelineDamageLog } from "./TimelineDamageLog";
import { checkFormLegality } from "../utils/formLegality";
import type { MijiDefDTO, QixueDefDTO } from "../types";
import styles from "./ComboPage.module.css";

const QIFENG_NAMES: Record<string, string> = {
  yangchun: "阳春白雪",
  gaoshan: "高山流水",
  pingsha: "平沙落雁",
};
const qifengName = (combat: CombatDataDTO, id: string): string => QIFENG_NAMES[id] ?? id ?? "—";

interface Props {
  xinfaName?: string;
  formData?: FormData | null;
  draft?: ComboDraft;
  onDraftChange?: Dispatch<SetStateAction<ComboDraft>>;
  configurationOnly?: boolean;
}

export default function ComboPage({ xinfaName, formData, draft: controlledDraft, onDraftChange, configurationOnly = false }: Props) {
  const [skillPool, setSkillPool] = useState<SkillPoolItemDTO[]>([]);
  const [localDraft, setLocalDraft] = usePersistentState(`${STORAGE_KEYS.comboDraft}:${encodeURIComponent(xinfaName ?? "mowen")}`, defaultComboDraft, comboDraftSchema);
  const draft = controlledDraft ?? localDraft;
  const setDraft = onDraftChange ?? setLocalDraft;
  const setDraftField = useCallback(<K extends keyof ComboDraft,>(key: K, update: SetStateAction<ComboDraft[K]>) => {
    setDraft((previous) => ({ ...previous, [key]: typeof update === "function"
      ? (update as (value: ComboDraft[K]) => ComboDraft[K])(previous[key]) : update }));
  }, [setDraft]);
  const { sequence, presetName, poolQuery } = draft;
  const collapsedGroups = useMemo(() => new Set(draft.collapsedGroups), [draft.collapsedGroups]);
  const setSequence = useCallback((update: SetStateAction<ComboStepDTO[]>) => setDraftField("sequence", update), [setDraftField]);
  const setPresetName = useCallback((value: string) => setDraftField("presetName", value), [setDraftField]);
  const setPoolQuery = useCallback((value: string) => setDraftField("poolQuery", value), [setDraftField]);
  const setCollapsedGroups = useCallback((update: SetStateAction<Set<string>>) => {
    setDraftField("collapsedGroups", (previous) => [...(typeof update === "function" ? update(new Set(previous)) : update)]);
  }, [setDraftField]);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const [presets, setPresets] = useState<string[]>([]);
  const [comboResult, setComboResult] = useState<ComboResultDTO | null>(null);
  const [timelineResult, setTimelineResult] = useState<TimelineResultDTO | null>(null);
  const [combatBundle, setCombatBundle] = useState<CombatBundleDTO | null>(null);
  const [combatError, setCombatError] = useState<string | null>(null);
  const legality = checkFormLegality(sequence, combatBundle?.combat);

  // #92 秘籍/奇穴：可选列表来自门派 combat overlay，勾选后随请求下发
  const [timelineMiji, setTimelineMiji] = useState<MijiDefDTO[]>([]);
  const [timelineQixue, setTimelineQixue] = useState<QixueDefDTO[]>([]);
  const [selMiji, setSelMiji] = useState<string[]>([]);
  const [selQixue, setSelQixue] = useState<string[]>([]);
  const [computing, setComputing] = useState(false);
  const [adjustTarget, setAdjustTarget] = useState<number | null>(null);
  const [saveDialog, setSaveDialog] = useState(false);
  /** 当前展开秘籍选择器的技能（key = skill_id:sub_id） */
  const [mijiPicker, setMijiPicker] = useState<string | null>(null);

  // 奇穴盘：第一重~第六重各选 1；第七~九重为混选池（最多 3 个）
  const qixueGroups = useMemo(() => {
    const singles = new Map<number, QixueDefDTO[]>();
    const mixed: QixueDefDTO[] = [];
    for (const q of timelineQixue) {
      if (q.slot > 6) {
        mixed.push(q);
      } else {
        const list = singles.get(q.slot) ?? [];
        list.push(q);
        singles.set(q.slot, list);
      }
    }
    const label = (slot: number) => `第${["一","二","三","四","五","六"][slot - 1]}重`;
    const groups = [...singles.entries()]
      .sort(([a], [b]) => a - b)
      .map(([slot, items]) => ({ slot, items, label: label(slot), mixed: false }));
    if (mixed.length > 0) groups.push({ slot: 7, items: mixed, label: "混选", mixed: true });
    return groups;
  }, [timelineQixue]);

  const toggleQixue = useCallback((slot: number, id: string, mixed: boolean) => {
    setSelQixue((prev) => {
      if (prev.includes(id)) return prev.filter((x) => x !== id);
      const sameSlot = new Set(timelineQixue.filter((q) => q.slot === slot).map((q) => q.id));
      const others = prev.filter((x) => !sameSlot.has(x));
      if (!mixed) return [...others, id];
      const mixedIds = new Set(timelineQixue.filter((q) => q.slot > 6).map((q) => q.id));
      if (others.filter((x) => mixedIds.has(x)).length >= 3) return prev;
      return [...others, id];
    });
  }, [timelineQixue]);
  useEffect(() => {
    let cancelled = false;
    setSkillPool([]);
    if (xinfaName) {
      api.loadSkillPool(xinfaName).then((pool) => { if (!cancelled) setSkillPool(pool); }).catch(() => {});
    }
    api.listComboPresets().then((names) => { if (!cancelled) setPresets(names); }).catch(() => {});
    const stored = readPreference(STORAGE_KEYS.favorites);
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
    return () => { cancelled = true; };
  }, [xinfaName]);

  const saveFavorites = useCallback((favs: Set<string>) => {
    setFavorites(favs);
    writePreference(STORAGE_KEYS.favorites, JSON.stringify([...favs]));
  }, []);

  const calculationRevision = useRef(0);
  useEffect(() => {
    ++calculationRevision.current;
    setComboResult(null);
    setComputing(false);
    return () => { ++calculationRevision.current; };
  }, [sequence, formData, selMiji, selQixue]);

  const pool = skillPool;

  // 技能唯一标识（同 skill_id 不同 sub_id 形态区分）
  const skillKey = useCallback((s: SkillPoolItemDTO) => `${s.skill_id}-${s.sub_id ?? 0}`, []);

  const addToSequence = useCallback((skill: SkillPoolItemDTO) => {
    setSequence((prev) => [...prev, { skill, overrides: null }]);
  }, [setSequence]);

  const removeFromSequence = useCallback((index: number) => {
    setSequence((prev) => prev.filter((_, i) => i !== index));
  }, [setSequence]);

  const clearSequence = useCallback(() => {
    setSequence([]);
    setComboResult(null);
  }, [setSequence]);

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
  }, [sequence, setSequence]);

  const updateOverride = useCallback((index: number, overrides: StepOverrideDTO) => {
    setSequence((prev) => prev.map((s, i) => i === index ? { ...s, overrides } : s));
  }, [setSequence]);

  const resetOverride = useCallback((index: number) => {
    setSequence((prev) => prev.map((s, i) => i === index ? { ...s, overrides: null } : s));
  }, [setSequence]);

  const handleCalculate = useCallback(async () => {
    if (sequence.length === 0 || !formData) return;
    const revision = ++calculationRevision.current;
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
        req.value_set,
      );
      if (revision === calculationRevision.current) setComboResult(result);
    } catch (err) {
      console.error(err);
    } finally {
      if (revision === calculationRevision.current) setComputing(false);
    }
  }, [sequence, formData, selMiji, selQixue]);

  // 面板属性同步后加载门派 combat overlay，填充秘籍/奇穴可选项
  useEffect(() => {
    if (!formData) return;
    let cancelled = false;
    void Promise.resolve()
      .then(() => api.loadCombatBundle(formData.xinfa_config.profession))
      .then((bundle) => {
        if (cancelled || !bundle?.combat) return;
        setCombatBundle(bundle);
        setCombatError(null);
        setTimelineMiji(bundle.combat.miji ?? []);
        setTimelineQixue(bundle.combat.qixue ?? []);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setCombatBundle(null);
        setCombatError(err instanceof Error ? err.message : String(err));
      });
    return () => {
      cancelled = true;
    };
  }, [formData?.xinfa_config.profession]);

  // #92 时间轴预览：读门派 combat overlay → DTO 请求 → 时序引擎（窗口伤害 / DPS / 事件）
  const handleTimeline = useCallback(async () => {
    if (sequence.length === 0 || !formData) return;
    setComputing(true);
    try {
      const bundle = await api.loadCombatBundle(formData.xinfa_config.profession);
      const req = toCalculateRequest(formData);
      const result = await api.calculateTimeline({
        skills: bundle.skills.length > 0 ? bundle.skills : skillPool,
        forms: bundle.combat.form,
        timed_buffs: bundle.combat.buff,
        actions: sequence.map((s) => ({
          kind: "cast" as const,
          skill_id: s.skill.skill_id,
          sub_id: s.skill.sub_id ?? 0,
          // 同一 (skill_id, sub_id) 可能对应多个变体（入月·N 层），必须带名字消歧
          skill_name: s.skill.skill_name,
        })),
        window: 300,
        player: req.player,
        hostile: req.hostile,
        xinfa: req.xinfa_config,
        buff: req.buff,
        coeff: req.coefficient,
        value_set: req.value_set ?? null,
        combat: bundle.combat,
        initial_instrument: bundle.combat.form.find((f) => f.initial)?.id ?? bundle.combat.form[0]?.id ?? null,
        initial_qifeng: bundle.combat.qifeng?.[0] ?? "",
        miji: selMiji,
        qixue: selQixue,
      });
      setTimelineResult(result);
    } catch (err) {
      console.error(err);
    } finally {
      setComputing(false);
    }
  }, [sequence, formData, selMiji, selQixue]);

  const handleSavePreset = useCallback(async () => {
    if (!presetName.trim()) return;
    try {
      await api.saveComboPreset(presetName.trim(), sequence);
      setDraftField("selectedPreset", presetName.trim());
      setSaveDialog(false);
      setPresets(await api.listComboPresets());
    } catch (err) {
      console.error(err);
    }
  }, [presetName, sequence, setDraftField]);

  const handleLoadPreset = useCallback(async (name: string) => {
    try {
      const preset = await api.loadComboPreset(name);
      setSequence(preset.steps);
      setPresetName(name);
      setDraftField("selectedPreset", name);
      setComboResult(null);
    } catch (err) {
      console.error(err);
    }
  }, [setSequence, setPresetName, setDraftField]);

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
  }, [setCollapsedGroups]);

  const expandAll = useCallback(() => setCollapsedGroups(new Set()), [setCollapsedGroups]);
  const collapseAll = useCallback(() => {
    setCollapsedGroups(new Set(groups.filter(([, items]) => items.length > 1).map(([name]) => name)));
  }, [groups, setCollapsedGroups]);

  // 面板最终攻击力（用于技能卡「攻击加成」展示）
  const attackValue = (() => {
    const player = (formData?.player ?? {}) as Record<string, number | string>;
    return Number(player.zuizhong_gongji ?? 0) || Number(player.jichu_gongji ?? 0) || 0;
  })();

  /** 某技能可选的秘籍（skill_id 匹配；sub_id=0 视为对所有变体生效） */
  const mijiFor = useCallback((skill: SkillPoolItemDTO) =>
    timelineMiji.filter((m) => m.skill_id === skill.skill_id
      && ((m.sub_id ?? 0) === 0 || (m.sub_id ?? 0) === (skill.sub_id ?? 0))),
  [timelineMiji]);

  /** 切换某技能的秘籍：同一技能最多 4 个（游戏内秘籍槽位） */
  const toggleMiji = useCallback((skill: SkillPoolItemDTO, id: string) => {
    setSelMiji((prev) => {
      if (prev.includes(id)) return prev.filter((x) => x !== id);
      const equipped = mijiFor(skill).filter((m) => prev.includes(m.id)).length;
      if (equipped >= 4) return prev;
      return [...prev, id];
    });
  }, [mijiFor]);

  /** 技能卡的时序标签：吟唱 / 调息 / 充能 */
  const castLabel = useCallback((skill: SkillPoolItemDTO) => {
    const parts: string[] = [];
    const cast = skill.cast_time ?? 0;
    parts.push(cast > 0 ? `${cast}s 吟唱` : "瞬发");
    if ((skill.charges ?? 0) > 1) parts.push(`${skill.charges} 层 · ${skill.recharge ?? 0}s`);
    else if ((skill.cooldown ?? 0) > 0) parts.push(`${skill.cooldown}s 调息`);
    return parts.join(" · ");
  }, []);

  /** 每一步的模拟时间、GCD 与其前的器形/曲风切换事件 */
  const stepMeta = useMemo(() => {
    type Switch = { switchKind: "form" | "qifeng"; from: string; to: string; time: number };
    const casts: { time: number; gcd: number }[] = [];
    const switches: Switch[] = [];
    for (const e of timelineResult?.events ?? []) {
      if (e.kind.kind === "cast") casts.push({ time: e.time, gcd: e.kind.gcd_secs ?? 0 });
      else if (e.kind.kind === "form_switch") switches.push({ switchKind: "form", from: e.kind.from, to: e.kind.to, time: e.time });
      else if (e.kind.kind === "qifeng_switch") switches.push({ switchKind: "qifeng", from: e.kind.from, to: e.kind.to, time: e.time });
    }
    let castIdx = 0;
    let swIdx = 0;
    return sequence.map((_, i) => {
      const legal = legality[i]?.legal ?? true;
      const cast = legal && castIdx < casts.length ? casts[castIdx] : null;
      const threshold = cast ? cast.time : Number.POSITIVE_INFINITY;
      const before: Switch[] = [];
      while (swIdx < switches.length && switches[swIdx].time <= threshold) {
        before.push(switches[swIdx]);
        swIdx += 1;
      }
      if (cast) castIdx += 1;
      return { time: cast ? cast.time : null, gcd: cast?.gcd ?? 0, switches: before };
    });
  }, [sequence, legality, timelineResult]);

  // 需先同步面板属性（未计算过时 formData 为 null，全 0 提交会让追加真伤恒为 0）
  const noForm = !formData;
  const targetHp = Number(formData?.hostile?.target_hp) || 0;
  const hasLostHpSkills = sequence.some((s) => (s.skill.lost_hp_zhenshishanghai ?? 0) > 0);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div className={styles.headerTitle}>
          <h2>武学</h2>
          <span className={styles.headerMeta}>逐招排轴 · 时间轴模拟</span>
        </div>
        <div className={styles.headerActions}>
          {sequence.length > 0 && (
            <>
              {!configurationOnly && <>
                <button className={styles.primaryBtn} onClick={handleCalculate} disabled={computing || noForm}
                  title={noForm ? "请先在「计算器」页输入属性并点击计算，同步面板与目标血量" : ""}>
                  {computing ? "计算中..." : "计算连招"}
                </button>
                <button className={styles.btn} onClick={handleTimeline}
                  disabled={computing || noForm || legality.some((s) => !s.legal)}
                  title={legality.some((s) => !s.legal)
                    ? "序列中存在当前形态不可用的技能，引擎会跳过；请先调整顺序或去掉这些技能"
                    : "按当前序列跑时间轴（GCD / 读条 / CD / 形态 / buff / dot）"}>
                  {computing ? "计算中..." : "时间轴预览"}
                </button>
              </>}
              <button className={styles.btn} onClick={clearSequence}>清空</button>
            </>
          )}
        </div>
      </div>

      {combatBundle && (
        <div className={styles.axisBar} data-testid="axis-bar">
          <span className={styles.axisChip}>
            器形 <b>{legality.length > 0 ? legality[legality.length - 1].form : (combatBundle.combat.form.find((f) => f.initial)?.name ?? "—")}</b>
          </span>
          <span className={styles.axisChip}>
            曲风 <b>{legality.length > 0 ? qifengName(combatBundle.combat, legality[legality.length - 1].qifeng) : qifengName(combatBundle.combat, combatBundle.combat.qifeng?.[0] ?? "")}</b>
          </span>
          {formData && (
            <>
              <span className={styles.axisDivider} />
              <span className={styles.axisChip}>
                目标 <b>{(Number(formData.hostile?.target_hp) || 0).toLocaleString()}</b> 血
              </span>
              <span className={styles.axisChip}>
                内防 <b>{(Number(formData.hostile?.neigong_fangyu) || 0).toLocaleString()}</b>
              </span>
              <span className={styles.axisChip}>
                减伤 <b>{((Number(formData.hostile?.jianshang_bili) || 0) * 100).toFixed(0)}%</b>
              </span>
            </>
          )}
          <span className={styles.axisDivider} />
          {combatBundle.combat.skill_combat
            .filter((sc) => sc.on_cast?.switch_qifeng)
            .map((sc) => {
              const sk = combatBundle.skills.find((k) => k.skill_id === sc.skill_id && (k.sub_id ?? 0) === (sc.sub_id ?? 0));
              if (!sk) return null;
              return (
                <button key={`${sc.skill_id}:${sc.sub_id ?? 0}`} type="button" className={styles.qifengBtn}
                  onClick={() => addToSequence(sk)}>
                  切至{qifengName(combatBundle.combat, sc.on_cast.switch_qifeng ?? "")}
                </button>
              );
            })}
        </div>
      )}

      {combatError && (
        <div className={styles.warnBanner}>
          战斗数据（data/combat）加载失败：{combatError}
        </div>
      )}

      {noForm && (
        <div className={styles.warnBanner}>
          {configurationOnly ? "请先在驾驶台完成配装或手动属性计算；连招草稿可继续编辑。" : "尚未同步面板属性：请先在「计算器」页输入属性并点击计算，再回来排轴（否则以 0 属性提交，追加真伤不会生效）。"}
        </div>
      )}
      {configurationOnly && <p className={styles.zhenshiHint}>草稿自动保存；切回计算驾驶台查看连招伤害与击杀率。</p>}
      {!configurationOnly && legality.some((s) => !s.legal) && (
        <div className={styles.warnBanner}>
          序列中有 {legality.filter((s) => !s.legal).length} 个技能在当前形态下不可用（已标红）；时间轴会跳过它们，建议调整顺序或换技能。
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

      <div className={styles.mainGrid}>
        {/* 左：技能目录 */}
        <aside className={styles.catalogPanel}>
          <div className={styles.panelTitle}>
            技能目录<span className={styles.panelHint}>点击加入排轴 · 右键收藏</span>
          </div>
          <div className={styles.presetBar}>
            <select className={styles.presetSelect} value={presets.includes(draft.selectedPreset) ? draft.selectedPreset : ""} onChange={(e) => e.target.value && handleLoadPreset(e.target.value)}>
              <option value="" disabled>加载预设...</option>
              {presets.map((p) => (
                <option key={p} value={p}>{p}</option>
              ))}
            </select>
            <button className={styles.smallBtn} onClick={() => setSaveDialog(true)}><IconSave size={14} /> 保存</button>
            {presets.length > 0 && (
              <button className={styles.smallBtn} onClick={() => { const p = presets[presets.length - 1]; handleDeletePreset(p); }} title="删除最近预设"><IconTrash size={14} /></button>
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
                      {items.map((s, itemIndex) => {
                        const key = skillKey(s);
                        const fav = favorites.has(key);
                        const def = combatBundle?.combat.skill_combat.find(
                          (sc) => sc.skill_id === s.skill_id && (sc.sub_id ?? 0) === (s.sub_id ?? 0),
                        );
                        const instrument = def?.requires_instrument ?? null;
                        const accent = instrument === "jian" ? styles.cardJian : styles.cardQin;
                        const qifengNeed = def?.requires_qifeng ?? null;
                        const miji = mijiFor(s);
                        const bonus = attackValue > 0 ? Math.round(s.atk_xishu * attackValue) : 0;
                        return (
                          <div key={`${key}-${itemIndex}`}
                            data-skill-card={key}
                            data-skill-name={s.skill_name}
                            className={`${styles.skillCard} ${accent} ${fav ? styles.favoriteCard : ""}`}
                            onClick={() => addToSequence(s)}
                            onContextMenu={(e) => { e.preventDefault(); toggleFavorite(s); }}
                            title={`${s.skill_name} | 基础伤害 ${s.base_damage1}-${s.base_damage2} | 系数 ${s.atk_xishu}${fav ? " · 已收藏" : " · 右键标记收藏"}`}>
                            {miji.length > 0 && <div className={styles.cardMijiSlots}>
                              {[0, 1, 2, 3].map((slot) => {
                                const equipped = miji.filter((m) => selMiji.includes(m.id))[slot];
                                return (
                                  <button
                                    key={slot}
                                    type="button"
                                    className={`${styles.mijiSlot} ${equipped ? styles.mijiSlotOn : ""}`}
                                    title={equipped ? `秘籍：${equipped.name}（点击卸下）` : `秘籍槽 ${slot + 1}（点击选择）`}
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      if (equipped) setSelMiji((prev) => prev.filter((x) => x !== equipped.id));
                                      else setMijiPicker(mijiPicker === key ? null : key);
                                    }}
                                  >
                                    {equipped ? equipped.name : "＋"}
                                  </button>
                                );
                              })}
                            </div>}
                            <div className={styles.cardBody}>
                            <div className={styles.cardHead}>
                              <span className={styles.cardName}>{s.skill_name}</span>
                              {qifengNeed && (
                                <span className={styles.cardBadge}>需{combatBundle ? qifengName(combatBundle.combat, qifengNeed) : qifengNeed}</span>
                              )}
                              <span className={styles.cardTag}>{castLabel(s)}</span>
                            </div>
                            <div className={styles.cardDamage}>
                              {s.base_damage1}–{s.base_damage2}
                              {bonus > 0 && <span className={styles.cardBonus}> (+{bonus.toLocaleString()})</span>}
                              {(s.dot_flag ?? 0) > 0 && <span className={styles.cardDot}> · 持续伤害</span>}
                              {s.lost_hp_zhenshishanghai > 0 && (
                                <span className={styles.comboZhenshiTag}>真伤{(s.lost_hp_zhenshishanghai * 100).toFixed(0)}%</span>
                              )}
                            </div>
                            {fav && <span className={styles.starIcon}><IconStar size={12} /></span>}
                            </div>
                            {mijiPicker === key && (
                              <div className={styles.mijiPicker} onClick={(e) => e.stopPropagation()}>
                                <div className={styles.mijiPickerHead}>
                                  <span>{s.skill_name} · 秘籍（{miji.filter((m) => selMiji.includes(m.id)).length}/4）</span>
                                  <button type="button" className={styles.mijiPickerClose} onClick={() => setMijiPicker(null)}>×</button>
                                </div>
                                <div className={styles.mijiPickerList}>
                                  {miji.map((m) => (
                                    <button key={m.id} type="button"
                                      className={`${styles.mijiOption} ${selMiji.includes(m.id) ? styles.mijiOptionOn : ""}`}
                                      onClick={() => toggleMiji(s, m.id)}>
                                      {m.name}
                                    </button>
                                  ))}
                                  {miji.length === 0 && <span className={styles.mijiEmpty}>该技能没有秘籍</span>}
                                </div>
                              </div>
                            )}
                          </div>
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
        </aside>

        {/* 中：排轴 */}
        <section className={styles.boardPanel}>
          <div className={styles.panelTitle}>
            排轴<span className={styles.panelHint}>拖拽排序 · 非法步骤会跳过</span>
          </div>
          <div className={styles.comboSequence}>
            {sequence.length === 0 ? (
              <span className={styles.emptyHint}>从下方技能池添加技能到序列</span>
            ) : (
              <DragDropContext onDragEnd={onDragEnd}>
                <Droppable droppableId="sequence" direction="vertical">
                  {(provided) => (
                    <div className={styles.sequenceRow} ref={provided.innerRef} {...provided.droppableProps}>
                      {sequence.map((item, i) => (
                        <Draggable key={`${item.skill.skill_id}-${item.skill.sub_id ?? 0}-${i}`} draggableId={`seq-${i}`} index={i}>
                          {(provided, snapshot) => (
                            <div ref={provided.innerRef} {...provided.draggableProps} {...provided.dragHandleProps}
                              className={`${styles.stepCard} ${snapshot.isDragging ? styles.dragging : ""} ${legality[i] && !legality[i].legal ? styles.illegalStep : ""}`}
                              title={legality[i] && !legality[i].legal ? `当前形态「${legality[i].form}」下不可用，需要「${legality[i].requiredForm}」` : undefined}>
                              {(stepMeta[i]?.switches ?? []).map((sw, si) => (
                                <div key={`sw-${si}`} className={styles.stepSwitch}>
                                  <span className={styles.stepSwitchKind}>{sw.switchKind === "form" ? "器形" : "曲风"}</span>
                                  <span className={styles.stepSwitchArrow}>⇄</span>
                                  <span className={styles.stepSwitchTo}>
                                    {sw.switchKind === "form"
                                      ? (combatBundle?.combat.form.find((f) => f.id === sw.to)?.name ?? sw.to)
                                      : (combatBundle ? qifengName(combatBundle.combat, sw.to) : sw.to)}
                                  </span>
                                </div>
                              ))}
                              <div className={styles.stepRow}>
                                <span className={styles.stepTime}>
                                  {stepMeta[i]?.time != null ? `${stepMeta[i].time!.toFixed(2)}s` : "—"}
                                </span>
                                <span className={styles.comboIndex}>{i + 1}</span>
                                <span className={styles.comboMain}>
                                  <span className={styles.comboName}>{item.skill.skill_name}</span>
                                  <span className={styles.comboMeta}>
                                    {legality[i]
                                      ? `${legality[i].form} · ${combatBundle ? qifengName(combatBundle.combat, legality[i].qifeng) : legality[i].qifeng}`
                                      : "—"}
                                  </span>
                                  {selMiji.length > 0 && (
                                    <span className={styles.stepMiji}>
                                      {selMiji
                                        .map((id) => timelineMiji.find((m) => m.id === id))
                                        .filter((m): m is NonNullable<typeof m> => !!m
                                          && m.skill_id === item.skill.skill_id
                                          && ((m.sub_id ?? 0) === 0 || (m.sub_id ?? 0) === (item.skill.sub_id ?? 0)))
                                        .map((m) => (
                                          <span key={m.id} className={styles.mijiPillOn}>{m.name}</span>
                                        ))}
                                    </span>
                                  )}
                                </span>
                                {legality[i] && !legality[i].legal && <span className={styles.illegalBadge}>非法</span>}
                                <span className={styles.stepGcd}>
                                  {stepMeta[i]?.gcd ? `GCD ${stepMeta[i].gcd.toFixed(2)}s` : ""}
                                </span>
                                <span className={styles.comboActions}>
                                  <button className={styles.smallBtn} onClick={() => setAdjustTarget(adjustTarget === i ? null : i)} title="调整系数"><IconGear size={14} /></button>
                                  <button className={styles.smallBtn} onClick={() => removeFromSequence(i)} title="移除"><IconClose size={14} /></button>
                                </span>
                              </div>
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
          {timelineResult && (
            <div className={styles.boardStats}>
              <span>当前序列 <b>{sequence.length}</b> 步</span>
              <span>形态/曲风切换 <b>{stepMeta.reduce((n, m) => n + m.switches.length, 0)}</b> 次</span>
              <span>非法步骤 <b>{legality.filter((s) => !s.legal).length}</b></span>
              <span>未执行动作 <b>{timelineResult.skipped_actions}</b></span>
            </div>
          )}
        </section>

        {/* 右：模拟伤害记录 */}
        <aside className={styles.logPanel}>
          <div className={styles.panelTitle}>
            模拟伤害记录<span className={styles.panelHint}>按时间轴结算顺序</span>
          </div>
          {timelineResult ? (
            <>
              <div className={styles.logScroll}>
                <TimelineDamageLog events={timelineResult.events} />
              </div>
              <div className={styles.totalsRow}>
                <div className={styles.totalBox}>
                  <b>{Math.round(timelineResult.total_damage).toLocaleString()}</b>
                  <span>窗口总伤</span>
                </div>
                <div className={styles.totalBox}>
                  <b>{timelineResult.window_dps.toFixed(0)}</b>
                  <span>DPS</span>
                </div>
                <div className={styles.totalBox}>
                  <b>{timelineResult.skipped_actions}</b>
                  <span>未执行动作</span>
                </div>
              </div>
            </>
          ) : (
            <div className={styles.logEmpty}>
              点「时间轴预览」后，这里按时间列出每次施放/持续伤害与累计伤害
            </div>
          )}
        </aside>
      </div>

      {/* 奇穴盘：同重互斥，第七~九重为混选池（最多 3 个） */}
      {timelineQixue.length > 0 && (
        <div className={styles.qixueBoard} data-testid="qixue-board">
          <div className={styles.qixueHead}>
            <span>奇穴盘</span>
            <span className={styles.qixueHint}>同重互斥 · 混选 9 选 3</span>
            <span className={styles.qixueCount}>已选 {selQixue.length} / 9</span>
          </div>
          <div className={styles.qixueGrid}>
            {qixueGroups.map((g) => (
              <div className={`${styles.qixueGroup} ${g.mixed ? styles.qixueGroupMixed : ""}`} key={g.slot}>
                <div className={styles.qixueGroupLabel}>
                  {g.label}
                  {g.mixed && <span className={styles.qixueGroupHint}>9 选 3</span>}
                </div>
                <div className={styles.qixueGroupChips}>
                  {g.items.map((q) => (
                    <button
                      key={q.id}
                      type="button"
                      className={`${styles.qixueChip} ${selQixue.includes(q.id) ? styles.qixueChipActive : ""}`}
                      onClick={() => toggleQixue(g.slot, q.id, g.mixed)}
                    >
                      {q.name}
                    </button>
                  ))}
                </div>
              </div>
            ))}

          </div>
        </div>
      )}

      {/* 连招结果（图表） */}
      {!configurationOnly && comboResult && <ComboResultDisplay result={comboResult} />}

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
