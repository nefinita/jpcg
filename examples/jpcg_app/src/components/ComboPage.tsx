import { buildTimelineRequest } from "../utils/timelineRequest";
import { useState, useEffect, useLayoutEffect, useCallback, useMemo, useRef, type SetStateAction, type Dispatch } from "react";
import { DragDropContext, Droppable, Draggable } from "@hello-pangea/dnd";
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import type {
  SkillPoolItemDTO, ComboStepDTO, StepOverrideDTO, SequenceItem, FormData, TimelineResultDTO, CombatBundleDTO, CombatDataDTO,
} from "../types";
import * as api from "../api/commands";
import { IconGear, IconClose, IconSave, IconTrash, IconStar } from "./icons";
import { usePersistentState, readPreference, writePreference } from "../hooks/usePersistentState";
import { STORAGE_KEYS } from "../utils/constants";
import { comboDraftSchema, defaultComboDraft, type ComboDraft } from "../state/cockpit";
import { formatTimelineEvent } from "./timelineFormat";
import { TimelineDamageLog } from "./TimelineDamageLog";
import type { MijiDefDTO, QixueDefDTO } from "../types";
import styles from "./ComboPage.module.css";

const QIFENG_NAMES: Record<string, string> = {
  yangchun: "阳春白雪",
  gaoshan: "高山流水",
  pingsha: "平沙落雁",
};
const qifengName = (combat: CombatDataDTO, id: string): string => QIFENG_NAMES[id] ?? id ?? "—";
type SkillStep = Extract<SequenceItem, { kind: "skill" }>;

interface Props {
  xinfaName?: string;
  formData?: FormData | null;
  draft?: ComboDraft;
  onDraftChange?: Dispatch<SetStateAction<ComboDraft>>;
  configurationOnly?: boolean;
}

export default function ComboPage({ xinfaName, formData, draft: controlledDraft, onDraftChange, configurationOnly = false }: Props) {
  const [localDraft, setLocalDraft] = usePersistentState(`${STORAGE_KEYS.comboDraft}:${encodeURIComponent(xinfaName ?? "mowen")}`, defaultComboDraft, comboDraftSchema);
  const draft = controlledDraft ?? localDraft;
  const setDraft = onDraftChange ?? setLocalDraft;
  const setDraftField = useCallback(<K extends keyof ComboDraft,>(key: K, update: SetStateAction<ComboDraft[K]>) => {
    setDraft((previous) => ({ ...previous, [key]: typeof update === "function"
      ? (update as (value: ComboDraft[K]) => ComboDraft[K])(previous[key]) : update }));
  }, [setDraft]);
  const { sequence, presetName, poolQuery } = draft;
  const collapsedGroups = useMemo(() => new Set(draft.collapsedGroups), [draft.collapsedGroups]);
  const setSequence = useCallback((update: SetStateAction<SequenceItem[]>) => setDraftField("sequence", update), [setDraftField]);
  const setPresetName = useCallback((value: string) => setDraftField("presetName", value), [setDraftField]);
  const setPoolQuery = useCallback((value: string) => setDraftField("poolQuery", value), [setDraftField]);
  const setCollapsedGroups = useCallback((update: SetStateAction<Set<string>>) => {
    setDraftField("collapsedGroups", (previous) => [...(typeof update === "function" ? update(new Set(previous)) : update)]);
  }, [setDraftField]);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const [presets, setPresets] = useState<string[]>([]);
  const [timelineResult, setTimelineResult] = useState<TimelineResultDTO | null>(null);
  const [combatBundle, setCombatBundle] = useState<CombatBundleDTO | null>(null);
  const [combatError, setCombatError] = useState<string | null>(null);
  // 步骤结果全部由后端给出（施放时刻 / GCD / 切换 / 合法性），前端只渲染
  const steps = timelineResult?.steps ?? [];
  const formName = useCallback(
    (id: string) => combatBundle?.combat.form.find((f) => f.id === id)?.name ?? id,
    [combatBundle],
  );

  // #92 秘籍/奇穴：可选列表来自门派 combat overlay；勾选结果写进连招草稿（持久化）
  const [timelineMiji, setTimelineMiji] = useState<MijiDefDTO[]>([]);
  const [timelineQixue, setTimelineQixue] = useState<QixueDefDTO[]>([]);
  const selMiji = draft.miji ?? [];
  const selQixue = draft.qixue ?? [];
  const setSelMiji = useCallback((update: SetStateAction<string[]>) =>
    setDraftField("miji", (previous) => typeof update === "function"
      ? (update as (value: string[]) => string[])(previous ?? [])
      : update), [setDraftField]);
  const setSelQixue = useCallback((update: SetStateAction<string[]>) =>
    setDraftField("qixue", (previous) => typeof update === "function"
      ? (update as (value: string[]) => string[])(previous ?? [])
      : update), [setDraftField]);
  const [adjustTarget, setAdjustTarget] = useState<number | null>(null);
  const [saveDialog, setSaveDialog] = useState(false);
  /** 秘籍配置弹窗（独立入口；左侧选技能，右侧配置 4 个槽） */
  const [mijiOpen, setMijiOpen] = useState(false);
  const [mijiSkill, setMijiSkill] = useState<SkillPoolItemDTO | null>(null);
  /** 技能目录过滤：全部 / 琴 / 剑 / 收藏 */
  const [poolFilter, setPoolFilter] = useState<"all" | "qin" | "jian" | "fav">("all");

  // 奇穴盘：分组/顺序/合并全部由后端算好（group_label / group_ids），前端只渲染。
  const qixueGroups = useMemo(() => {
    type Chip = { key: string; label: string; ids: string[]; shelved: boolean; slot: number };
    const mixedFrom = combatBundle?.combat.meta.mixed_slot_from ?? 7;
    const groups: { label: string; mixed: boolean; chips: Chip[] }[] = [];
    for (const q of timelineQixue) {
      const label = q.group_label || `第${["一", "二", "三", "四", "五", "六"][q.slot - 1] ?? "?"}重`;
      let group = groups[groups.length - 1];
      if (!group || group.label !== label) {
        group = { label, mixed: q.slot >= mixedFrom, chips: [] };
        groups.push(group);
      }
      group.chips.push({
        key: q.id,
        label: q.name,
        ids: q.group_ids?.length ? q.group_ids : [q.id],
        shelved: q.shelved ?? false,
        slot: q.slot,
      });
    }
    return groups;
  }, [timelineQixue, combatBundle]);
  const selectedQixueCount = qixueGroups.reduce(
    (n, g) => n + g.chips.filter((c) => c.ids.some((id) => selQixue.includes(id))).length,
    0,
  );

  const toggleQixue = useCallback((slot: number, ids: string[], mixed: boolean) => {
    setSelQixue((prev) => {
      const selected = ids.some((id) => prev.includes(id));
      if (selected) return prev.filter((x) => !ids.includes(x));
      if (!mixed) {
        // 固定重数：同一重内互斥
        const sameSlot = new Set(timelineQixue.filter((q) => q.slot === slot).flatMap((q) => q.group_ids?.length ? q.group_ids : [q.id]));
        return [...prev.filter((x) => !sameSlot.has(x)), ...ids];
      }
      // 混选池：上限由后端 meta 提供，按「格」计数（后端已把同一奇穴多条效果合并）
      const mixedFrom = combatBundle?.combat.meta.mixed_slot_from ?? 7;
      const cap = combatBundle?.combat.meta.mixed_qixue_cap ?? 3;
      const next = [...prev, ...ids];
      const picked = timelineQixue.filter(
        (q) => q.slot >= mixedFrom && (q.group_ids ?? [q.id]).some((id) => next.includes(id)),
      ).length;
      if (picked > cap) return prev;
      return next;
    });
  }, [timelineQixue, combatBundle]);

  useEffect(() => {
    let cancelled = false;
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

  // 技能目录直接用后端打包好的 bundle.skills（已套 overlay），前端不再另取技能池

  const pool = combatBundle?.skills ?? [];

  // 技能唯一标识（同 skill_id 不同 sub_id 形态区分）
  const skillKey = useCallback((s: SkillPoolItemDTO) => `${s.skill_id}-${s.sub_id ?? 0}`, []);

  const addToSequence = useCallback((skill: SkillPoolItemDTO) => {
    setSequence((prev) => [...prev, { kind: "skill", skill, overrides: null }]);
  }, [setSequence]);

  const addFlag = useCallback((flag: "start" | "end") => {
    setSequence((prev) => (prev.some((s) => s.kind === "flag" && s.flag === flag)
      ? prev
      : [...prev, { kind: "flag", flag }]));
  }, [setSequence]);

  const removeFromSequence = useCallback((index: number) => {
    setSequence((prev) => prev.filter((_, i) => i !== index));
  }, [setSequence]);

  const clearSequence = useCallback(() => {
    setSequence([]);
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
    setSequence((prev) => prev.map((s, i) => (i === index && s.kind === "skill" ? { ...s, overrides } : s)));
  }, [setSequence]);

  const resetOverride = useCallback((index: number) => {
    setSequence((prev) => prev.map((s, i) => (i === index && s.kind === "skill" ? { ...s, overrides: null } : s)));
  }, [setSequence]);

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

  // #92 时间轴模拟：读门派 combat overlay → DTO 请求 → 时序引擎（窗口伤害 / DPS / 事件）
  // 改动后自动重算（防抖 + revision 防旧结果覆盖），不再需要手动点按钮。
  const timelineRevision = useRef(0);
  const [timelineBusy, setTimelineBusy] = useState(false);
  const [showCurve, setShowCurve] = useState(false);
  const [timelineError, setTimelineError] = useState<string | null>(null);

  const runTimeline = useCallback(async () => {
    if (!formData) return;
    const revision = ++timelineRevision.current;
    setTimelineBusy(true);
    setTimelineError(null);
    setTimelineResult(null);
    try {
      const bundle = await api.loadCombatBundle(formData.xinfa_config.profession);
      const result = await api.calculateTimeline(buildTimelineRequest(formData, bundle, sequence, selMiji, selQixue, 50000));
      if (revision === timelineRevision.current) setTimelineResult(result);
    } catch (err) {
      if (revision === timelineRevision.current) setTimelineError(err instanceof Error ? err.message : String(err));
    } finally {
      if (revision === timelineRevision.current) setTimelineBusy(false);
    }
  }, [formData, combatBundle, sequence, selMiji, selQixue]);

  useLayoutEffect(() => {
    timelineRevision.current += 1;
    setTimelineResult(null);
    setTimelineError(null);
    setTimelineBusy(Boolean(formData && sequence.length));
    if (!formData || sequence.length === 0) {
      setTimelineResult(null);
      return;
    }
    const timer = setTimeout(() => { void runTimeline(); }, 350);
    return () => {
      clearTimeout(timer);
      timelineRevision.current += 1;
    };
  }, [runTimeline, formData, sequence.length]);

  const handleSavePreset = useCallback(async () => {
    if (!presetName.trim()) return;
    try {
      let position = 0;
      const flags: {position: number; start: boolean}[] = [];
      for (const item of sequence) {
        if (item.kind === "flag") flags.push({position, start: item.flag === "start"});
        else position++;
      }
      await api.saveComboPreset(presetName.trim(), sequence.filter((s): s is Extract<SequenceItem, { kind: "skill" }> => s.kind === "skill").map((s) => ({ skill: s.skill, overrides: s.overrides })), flags);
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
      const restored: SequenceItem[] = [];
      for (let i = 0; i <= preset.steps.length; i++) {
        for (const flag of preset.flags ?? []) if (flag.position === i) restored.push({kind: "flag", flag: flag.start ? "start" : "end"});
        const step = preset.steps[i];
        if (step) restored.push({kind: "skill", skill: step.skill, overrides: step.overrides});
      }
      setSequence(restored);
      setPresetName(name);
      setDraftField("selectedPreset", name);
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

  /** 某技能可选的秘籍：后端已在 SkillCombatDTO.miji_ids 里配对好，前端只做 id → 数据查找 */
  const mijiFor = useCallback((skill: SkillPoolItemDTO) => {
    const def = combatBundle?.combat.skill_combat.find(
      (sc) => sc.skill_id === skill.skill_id && (sc.sub_id ?? 0) === (skill.sub_id ?? 0),
    );
    return (def?.miji_ids ?? [])
      .map((id) => timelineMiji.find((m) => m.id === id))
      .filter((m): m is NonNullable<typeof m> => !!m);
  }, [combatBundle, timelineMiji]);

  /** 秘籍效果文案（后端生成，前端只拼接展示） */
  const mijiText = useCallback((m: MijiDefDTO) =>
    m.effects && m.effects.length > 0 ? m.effects.join("，") : "无数值效果", []);

  /** 切换某技能的秘籍：同一技能最多 4 个（游戏内秘籍槽位） */
  const mijiSkills = useMemo(() => pool.filter((s) => mijiFor(s).length > 0), [pool, mijiFor]);

  const toggleMiji = useCallback((skill: SkillPoolItemDTO, id: string) => {
    setSelMiji((prev) => {
      if (prev.includes(id)) return prev.filter((x) => x !== id);
      const cap = combatBundle?.combat.meta.miji_per_skill_cap ?? 4;
      const equipped = mijiFor(skill).filter((m) => prev.includes(m.id)).length;
      if (equipped >= cap) return prev;
      return [...prev, id];
    });
  }, [mijiFor, combatBundle]);

  /** 技能卡的时序标签：吟唱 / 调息 / 充能 */
  const castLabel = useCallback((skill: SkillPoolItemDTO) => {
    const parts: string[] = [];
    const cast = skill.cast_time ?? 0;
    parts.push(cast > 0 ? `${cast}s 吟唱` : "瞬发");
    if ((skill.charges ?? 0) > 1) parts.push(`${skill.charges} 层 · ${skill.recharge ?? 0}s`);
    else if ((skill.cooldown ?? 0) > 0) parts.push(`${skill.cooldown}s 调息`);
    return parts.join(" · ");
  }, []);

  const tileSkills = useMemo(() => {
    const all = groups
      .flatMap(([, items]) => items)
      // 奇穴门控技能：未激活对应奇穴时不出现在技能目录
      .filter((s) => {
        const def = combatBundle?.combat.skill_combat.find(
          (sc) => sc.skill_id === s.skill_id && (sc.sub_id ?? 0) === (s.sub_id ?? 0),
        );
        // 后端已把门控奇穴解析成 qixue id，前端只做激活集合判断
        const gate = def?.jihuoqixue_id ?? def?.jihuoqixue;
        return !gate || selQixue.includes(gate);
      });
    if (poolFilter === "all") return all;
    if (poolFilter === "fav") return all.filter((s) => favorites.has(skillKey(s)));
    return all.filter((s) => {
      const def = combatBundle?.combat.skill_combat.find(
        (sc) => sc.skill_id === s.skill_id && (sc.sub_id ?? 0) === (s.sub_id ?? 0),
      );
      const instrument = def?.requires_instrument ?? "qin";
      return instrument === poolFilter;
    });
  }, [groups, poolFilter, favorites, skillKey, combatBundle, selQixue]);


  const activeMijiSkill = mijiSkill ?? mijiSkills[0] ?? null;

  // 需先同步面板属性（未计算过时 formData 为 null，全 0 提交会让追加真伤恒为 0）
  const noForm = !formData;
  const targetHp = Number(formData?.hostile?.target_hp) || 0;
  const hasLostHpSkills = sequence.some((s) => s.kind === "skill" && (s.skill.lost_hp_zhenshishanghai ?? 0) > 0);
  const hasStartFlag = sequence.some((s) => s.kind === "flag" && s.flag === "start");
  const hasEndFlag = sequence.some((s) => s.kind === "flag" && s.flag === "end");
  const curveData = (timelineResult?.kill_prob_curve ?? []).map(([step, prob]) => ({
    step: `第${step}招`,
    prob: Math.round(prob * 10000) / 100,
  }));

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div className={styles.headerTitle}>
          <h2>武学</h2>
          <span className={styles.headerMeta}>逐招排轴 · 时间轴模拟</span>
        </div>
        <div className={styles.headerActions}>
          <button className={styles.btn} disabled={hasStartFlag} onClick={() => addFlag("start")}
            title="计算开始：击杀率从下一个施放起手算起">计算开始</button>
          <button className={styles.btn} disabled={hasEndFlag} onClick={() => addFlag("end")}
            title="计算结束：击杀率算到下一个施放起手为止">计算结束</button>
          {sequence.length > 0 && (
            <>
              {timelineBusy && <span className={styles.autoBadge}>自动模拟中…</span>}
              <button className={styles.btn} onClick={clearSequence}>清空</button>
            </>
          )}
        </div>
      </div>

      {combatBundle && (
        <div className={styles.axisBar} data-testid="axis-bar">
          <span className={styles.axisChip}>
            器形 <b>{formName(timelineResult?.final_instrument || (combatBundle.combat.form.find((f) => f.initial)?.id ?? ""))}</b>
          </span>
          <span className={styles.axisChip}>
            曲风 <b>{qifengName(combatBundle.combat, timelineResult?.final_qifeng || combatBundle.combat.qifeng?.[0] || "")}</b>
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
                减伤 <b>{(Number(formData.hostile?.jianshang_bili) || 0).toFixed(0)}%</b>
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
      {!configurationOnly && steps.some((step) => !step.legal) && (
        <div className={styles.warnBanner}>
          序列中有 {steps.filter((step) => !step.legal).length} 个技能在当前形态下不可用（已标红）；时间轴会跳过它们，建议调整顺序或换技能。
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
          <div className={styles.catalogHeader}>
            <div className={styles.panelTitle}>
              技能目录<span className={styles.panelHint}>点击加入排轴 · 右键收藏</span>
            </div>
            <button type="button" className={styles.mijiEntryBtn}
              onClick={() => { setMijiSkill(null); setMijiOpen(true); }}>
              秘籍配置
            </button>
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
          </div>
          <div className={styles.filterRow}>
            {([["all", "全部"], ["qin", "曲"], ["jian", "剑"], ["fav", "已收藏"]] as const).map(([key, label]) => (
              <button key={key} type="button"
                className={`${styles.filterChip} ${poolFilter === key ? styles.filterChipOn : ""}`}
                onClick={() => setPoolFilter(key)}>
                {label}
              </button>
            ))}
          </div>
          <div className={styles.comboPool}>
            <div className={styles.tileGrid}>
              {tileSkills.map((s, itemIndex) => {
                const key = skillKey(s);
                const fav = favorites.has(key);
                const def = combatBundle?.combat.skill_combat.find(
                  (sc) => sc.skill_id === s.skill_id && (sc.sub_id ?? 0) === (s.sub_id ?? 0),
                );
                const accent = def?.requires_instrument === "jian" ? styles.tileJian : styles.tileQin;
                const qifengNeed = def?.requires_qifeng ?? null;
                const miji = mijiFor(s);
                const equipped = miji.filter((m) => selMiji.includes(m.id)).length;
                return (
                  <div key={`${key}-${itemIndex}`}
                    className={`${styles.skillTile} ${accent} ${fav ? styles.favoriteTile : ""}`}
                    onContextMenu={(e) => { e.preventDefault(); toggleFavorite(s); }}
                    title={`${s.skill_name} | ${s.base_damage1}-${s.base_damage2}${qifengNeed ? ` | 需${qifengNeed}` : ""}${fav ? " · 已收藏" : " · 右键收藏"}`}>
                    <button type="button" className={styles.tileAdd} data-skill-card={key}
                      data-skill-name={s.skill_name} onClick={() => addToSequence(s)} aria-label={`添加${s.skill_name}`}>
                    <span className={styles.tileName}>{s.skill_name}</span>
                    <span className={styles.tileTag}>{castLabel(s)}</span>
                    </button>
                    {miji.length > 0 && (
                      <button type="button" className={`${styles.tileGear} ${equipped > 0 ? styles.tileGearOn : ""}`}
                        title="秘籍配置"
                        onClick={(e) => { e.stopPropagation(); setMijiSkill(s); setMijiOpen(true); }}>
                        {equipped}/4
                      </button>
                    )}
                    {fav && <span className={styles.tileStar}><IconStar size={10} /></span>}
                  </div>
                );
              })}
              {groups.length === 0 && (
                <span className={styles.emptyHint}>{pool.length === 0 ? "先选择心法加载技能池" : "没有匹配的技能"}</span>
              )}
            </div>
          </div>
        </aside>

        {/* 中：排轴 */}
        <section className={styles.boardPanel}>
          <div className={styles.panelTitle}>
            排轴<span className={styles.panelHint}>拖拽排序 · 非法步骤会跳过</span>
          </div>
          <div className={styles.comboSequence}>
            {sequence.length === 0 ? (
              <span className={styles.emptyHint}>从左侧技能目录添加技能到序列</span>
            ) : (
              <DragDropContext onDragEnd={onDragEnd}>
                <Droppable droppableId="sequence" direction="vertical">
                  {(provided) => (
                    <div className={styles.sequenceRow} ref={provided.innerRef} {...provided.droppableProps}>
                      {sequence.map((item, i) => (item.kind === "flag" ? (
                        <Draggable key={`flag-${item.flag}-${i}`} draggableId={`seq-${i}`} index={i}>
                          {(provided, snapshot) => (
                            <div ref={provided.innerRef} {...provided.draggableProps} {...provided.dragHandleProps}
                              className={`${styles.stepCard} ${styles.flagCard} ${snapshot.isDragging ? styles.dragging : ""}`}>
                              <span className={styles.flagLabel}>{item.flag === "start" ? "计算开始" : "计算结束"}</span>
                              <span className={styles.stepTime}>{steps[i]?.time != null ? `${steps[i].time!.toFixed(2)}s` : "—"}</span>
                              <button className={styles.smallBtn} onClick={() => removeFromSequence(i)} title="移除"><IconClose size={14} /></button>
                            </div>
                          )}
                        </Draggable>
                      ) : (
                        <Draggable key={`${item.skill.skill_id}-${item.skill.sub_id ?? 0}-${i}`} draggableId={`seq-${i}`} index={i}>
                          {(provided, snapshot) => (
                            <div ref={provided.innerRef} {...provided.draggableProps} {...provided.dragHandleProps}
                              className={`${styles.stepCard} ${snapshot.isDragging ? styles.dragging : ""} ${steps[i] && !steps[i].legal ? styles.illegalStep : ""}`}
                              title={steps[i] && !steps[i].legal ? `${steps[i].skipped_reason}${steps[i].required_instrument ? `，需要器形「${formName(steps[i].required_instrument!)}」` : ""}${steps[i].required_qifeng ? `，需要曲风「${qifengName(combatBundle?.combat ?? ({} as never), steps[i].required_qifeng!)}」` : ""}` : undefined}>
                              {(steps[i]?.switches ?? []).map((sw, si) => (
                                <div key={`sw-${si}`} className={styles.stepSwitch}>
                                  <span className={styles.stepSwitchKind}>{sw.switch_kind === "form" ? "器形" : "曲风"}</span>
                                  <span className={styles.stepSwitchArrow}>⇄</span>
                                  <span className={styles.stepSwitchTo}>
                                    {sw.switch_kind === "form" ? formName(sw.to) : qifengName(combatBundle?.combat ?? ({} as never), sw.to)}
                                  </span>
                                </div>
                              ))}
                              <div className={styles.stepRow}>
                                <span className={styles.stepTime}>
                                  {steps[i]?.time != null ? `${steps[i].time!.toFixed(2)}s` : "—"}
                                </span>
                                <span className={styles.comboIndex}>{i + 1}</span>
                                <span className={styles.comboMain}>
                                  <span className={styles.comboName}>{item.skill.skill_name}</span>
                                  <span className={styles.comboMeta}>
                                    {steps[i]
                                      ? `${formName(steps[i].instrument)} · ${qifengName(combatBundle?.combat ?? ({} as never), steps[i].qifeng)}`
                                      : "—"}
                                  </span>
                                  {selMiji.length > 0 && (
                                    <span className={styles.stepMiji}>
                                      {mijiFor(item.skill)
                                        .filter((m) => selMiji.includes(m.id))
                                        .map((m) => (
                                          <span key={m.id} className={styles.mijiPillOn}>{mijiText(m)}</span>
                                        ))}
                                    </span>
                                  )}
                                </span>
                                {steps[i] && !steps[i].legal && <span className={styles.illegalBadge}>非法</span>}
                                <span className={styles.stepGcd}>
                                  {steps[i]?.gcd_secs ? `GCD ${steps[i].gcd_secs.toFixed(2)}s` : ""}
                                </span>
                                <span className={styles.comboActions}>
                                  <button className={styles.smallBtn} onClick={() => setAdjustTarget(adjustTarget === i ? null : i)} title="调整系数"><IconGear size={14} /></button>
                                  <button className={styles.smallBtn} onClick={() => removeFromSequence(i)} title="移除"><IconClose size={14} /></button>
                                </span>
                              </div>
                            </div>
                          )}
                        </Draggable>
                      )))}
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
              <span>形态/曲风切换 <b>{steps.reduce((n, step) => n + step.switches.length, 0)}</b> 次</span>
              <span>非法步骤 <b>{steps.filter((step) => !step.legal).length}</b></span>
              <span>未执行动作 <b>{timelineResult.skipped_actions}</b></span>
            </div>
          )}
        </section>

        {/* 右：模拟伤害记录 */}
        <aside className={styles.logPanel}>
          <div className={styles.panelTitle}>
            模拟伤害记录<span className={styles.panelHint}>按时间轴结算顺序</span>
          </div>
          {timelineError ? (
            <div role="alert" className={styles.logEmpty}>
              <p>模拟失败：{timelineError}</p>
              <button type="button" onClick={() => void runTimeline()}>重试模拟</button>
            </div>
          ) : timelineResult ? (
            <>
              <div className={styles.logScroll}>
                <TimelineDamageLog events={timelineResult.events} killStart={timelineResult.kill_start} killEnd={timelineResult.kill_end} />
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
                  <b>{timelineResult.kill_start != null && timelineResult.kill_end != null
                    ? `${timelineResult.kill_start.toFixed(1)}–${timelineResult.kill_end.toFixed(1)}s`
                    : "—"}</b>
                  <span>测量窗口</span>
                </div>
                <div className={styles.totalBox}>
                  <b>{timelineResult.skipped_actions}</b>
                  <span>未执行动作</span>
                </div>
                <div className={styles.totalBox}>
                  <b>{timelineResult.kill_prob_curve && timelineResult.kill_prob_curve.length > 0
                    ? `${((timelineResult.final_kill_prob ?? 0) * 100).toFixed(1)}%`
                    : "—"}</b>
                  <span>击杀率</span>
                </div>
                <button type="button" className={styles.killBtn}
                  disabled={!timelineResult.kill_prob_curve || timelineResult.kill_prob_curve.length === 0}
                  onClick={() => setShowCurve(true)}>
                  曲线
                </button>
              </div>
            </>
          ) : (
            <div className={styles.logEmpty}>
              排轴、秘籍或奇穴一变，这里会自动按时间列出每次施放/持续伤害与累计伤害
            </div>
          )}
        </aside>
      </div>

      {/* 奇穴盘：同重互斥，第七~九重为混选池（最多 3 个） */}
      {timelineQixue.length > 0 && (
        <details open className={styles.qixueBoard} data-testid="qixue-board">
          <summary className={styles.qixueHead}>
            <span>奇穴盘</span>
            <span className={styles.qixueHint}>同重互斥 · 混选 9 选 3</span>
            <span className={styles.qixueLegend}>
              <i className={styles.qixueLegendDot} />
              虚线 = 引擎未实装，暂不可选
            </span>
            <span className={styles.qixueCount}>已选 {selectedQixueCount} · 点击收起/展开</span>
          </summary>
          <div className={styles.qixueGrid}>
            {qixueGroups.map((g) => (
              <div className={`${styles.qixueGroup} ${g.mixed ? styles.qixueGroupMixed : ""}`} key={g.label}>
                <div className={styles.qixueGroupLabel}>
                  {g.label}
                  {g.mixed && <span className={styles.qixueGroupHint}>9 选 3</span>}
                </div>
                <div className={styles.qixueGroupChips}>
                  {g.chips.map((chip) => {
                    const active = chip.ids.some((id) => selQixue.includes(id));
                    return (
                      <button
                        key={chip.key}
                        type="button"
                        className={`${styles.qixueChip} ${active ? styles.qixueChipActive : ""} ${chip.shelved ? styles.qixueChipShelved : ""}`}
                        aria-pressed={active}
                        disabled={chip.shelved && !active}
                        title={chip.shelved ? "引擎暂未建模；已保存的选择请取消" : undefined}
                        onClick={() => toggleQixue(chip.slot, chip.ids, g.mixed)}
                      >
                        {chip.label}
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}

          </div>
        </details>
      )}

      {/* 击杀概率曲线（悬浮卡片） */}
      {showCurve && (
        <div className={styles.curveOverlay} onClick={() => setShowCurve(false)}>
          <div className={styles.curveCard} onClick={(e) => e.stopPropagation()}>
            <div className={styles.curveCardHeader}>
              <span>击杀概率曲线</span>
              <button type="button" className={styles.btn} onClick={() => setShowCurve(false)}>关闭</button>
            </div>
            <div className={styles.chartContainer}>
              <ResponsiveContainer width="100%" height={220}>
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
        </div>
      )}

      {/* 秘籍配置（独立入口：左侧技能，右侧 4 个槽位 + 候选） */}
      {mijiOpen && (
        <div className={styles.mijiOverlay} onClick={() => setMijiOpen(false)}>
          <div className={styles.mijiModal} onClick={(e) => e.stopPropagation()} data-testid="miji-config">
            <div className={styles.mijiModalHeader}>
              <span>秘籍配置</span>
              <button onClick={() => setMijiOpen(false)} aria-label="关闭">×</button>
            </div>
            <div className={styles.mijiModalBody}>
              <div className={styles.mijiLayout}>
                <div className={styles.mijiSkillList}>
                  {mijiSkills.map((s) => {
                    const active = activeMijiSkill
                      && activeMijiSkill.skill_id === s.skill_id
                      && (activeMijiSkill.sub_id ?? 0) === (s.sub_id ?? 0);
                    const count = mijiFor(s).filter((m) => selMiji.includes(m.id)).length;
                    return (
                      <button key={skillKey(s)} type="button"
                        className={`${styles.mijiSkillItem} ${active ? styles.mijiSkillItemOn : ""}`}
                        onClick={() => setMijiSkill(s)}>
                        <span>{s.skill_name}</span>
                        <span className={styles.mijiSkillCount}>{count}/4</span>
                      </button>
                    );
                  })}
                  {mijiSkills.length === 0 && <span className={styles.mijiEmpty}>没有可配置秘籍的技能</span>}
                </div>
                <div className={styles.mijiDetail}>
                  {activeMijiSkill ? (
                    <>
                      <div className={styles.mijiDetailTitle}>
                        {activeMijiSkill.skill_name}
                        <span>{mijiFor(activeMijiSkill).filter((m) => selMiji.includes(m.id)).length}/4</span>
                      </div>
                      <div className={styles.mijiConfigSlots}>
                        {[0, 1, 2, 3].map((slot) => {
                          const equipped = mijiFor(activeMijiSkill).filter((m) => selMiji.includes(m.id))[slot];
                          return (
                            <span key={slot} className={`${styles.mijiConfigSlot} ${equipped ? styles.mijiConfigSlotOn : ""}`}>
                              {equipped ? mijiText(equipped) : `空槽 ${slot + 1}`}
                            </span>
                          );
                        })}
                      </div>
                      <div className={styles.mijiConfigList}>
                        {mijiFor(activeMijiSkill).map((m) => (
                          <button key={m.id} type="button"
                            className={`${styles.mijiOption} ${selMiji.includes(m.id) ? styles.mijiOptionOn : ""}`}
                            onClick={() => toggleMiji(activeMijiSkill, m.id)}>
                            {mijiText(m)}
                          </button>
                        ))}
                      </div>
                    </>
                  ) : (
                    <span className={styles.mijiEmpty}>选择左侧技能配置秘籍</span>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Step adjust modal */}
      {adjustTarget !== null && (
        <StepAdjustModal
          step={sequence[adjustTarget] as SkillStep}
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
  step: SkillStep; index: number;
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
