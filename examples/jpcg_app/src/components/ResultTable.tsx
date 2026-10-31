import { useState, useEffect, useRef, memo } from "react";
import type { SkillResultDTO } from "../types";
import { formatNumber } from "../utils/format";
import clsx from "../utils/clsx";
import AnimatedNumber from "./AnimatedNumber";
import styles from "./ResultTable.module.css";

interface Props {
  results: SkillResultDTO[] | null;
  calculating: boolean;
  baseline?: SkillResultDTO[];
}

function ResultTable({ results, calculating, baseline }: Props) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState("original");
  const previousResults = useRef<SkillResultDTO[] | null>(null);
  const previous = previousResults.current;
  useEffect(() => { previousResults.current = results; }, [results]);

  if (calculating && !results?.length) {
    return (
      <div className={styles.card}>
        <div className={styles.header}>
          <span>计算结果</span>
          <span style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>计算中...</span>
        </div>
        <div className={styles.skeleton}>
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className={styles.skeletonRow} style={{ width: `${70 + i * 5}%` }} />
          ))}
        </div>
      </div>
    );
  }

  if (!results || results.length === 0) {
    return (
      <div className={styles.card}>
        <div className={styles.header}>
          <span>计算结果</span>
        </div>
        <div className={styles.empty}>
          配置属性后点击「计算」查看伤害
        </div>
      </div>
    );
  }

  const totalQ = results.reduce((sum, result) => sum + result.q, 0);
  const maxQ = Math.max(...results.map((r) => r.q));
  const avgQ = Math.round(results.reduce((s, r) => s + r.q, 0) / results.length);
  const critSkills = results.filter((r) => r.h > r.n);
  const critRatio = results.length > 0 ? critSkills.length / results.length : 0;

  // 只改变展示顺序，不修改计算响应；同值维持原顺序，DOT 跳序始终成组保留。
  const keyword = query.trim().toLocaleLowerCase();
  // 旧结果 DTO 无技能 id：按名称与原始出现序匹配，排序不影响基准对应。
  const occurrences = new Map<string, number>();
  const identified = results.map((r) => {
    const occurrence = occurrences.get(r.skill_name) ?? 0;
    occurrences.set(r.skill_name, occurrence + 1);
    return { ...r, occurrence };
  });
  const baselineByName = new Map<string, SkillResultDTO[]>();
  for (const skill of baseline ?? []) {
    const group = baselineByName.get(skill.skill_name) ?? [];
    group.push(skill);
    baselineByName.set(skill.skill_name, group);
  }
  const visibleSkills = identified.filter((r) => r.skill_name.toLocaleLowerCase().includes(keyword));
  if (sort === "q-desc") visibleSkills.sort((a, b) => b.q - a.q);
  if (sort === "q-asc") visibleSkills.sort((a, b) => a.q - b.q);
  if (sort === "name") visibleSkills.sort((a, b) => a.skill_name.localeCompare(b.skill_name, "zh-CN"));

  // DOT 技能展开为每跳一行（dotIndex = 跳序 0..n-1）；非 DOT 技能保持原行
  const rows = visibleSkills.flatMap((r) => {
    const jumps = r.dot_jumps ?? [];
    if (jumps.length === 0) return [{ ...r, dotIndex: null as number | null }];
    return jumps.map((_, k) => ({ ...r, dotIndex: k }));
  });

  return (
    <div className={styles.card}>
      <div className={styles.header}>
        <span>计算结果</span>
        <span role="status" aria-atomic="true" style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>
          {calculating ? "计算中 · " : ""}{visibleSkills.length} / {results.length} 个技能
        </span>
      </div>
      <div className={styles.controls}>
        <label>
          搜索技能
          <input type="search" value={query} placeholder="输入技能名称"
            onChange={(event) => setQuery(event.target.value)} />
        </label>
        <label>
          排序
          <select value={sort} onChange={(event) => setSort(event.target.value)}>
            <option value="original">原始顺序</option>
            <option value="q-desc">技能期望 Q（合计）：从高到低</option>
            <option value="q-asc">技能期望 Q（合计）：从低到高</option>
            <option value="name">技能名称</option>
          </select>
        </label>
        <button type="button" onClick={() => { setQuery(""); setSort("original"); }}
          disabled={query === "" && sort === "original"}>重置筛选与排序</button>
        <span className={styles.hint}>按「技能合计 Q」排序（DOT 行显示单跳 Q，各跳保持连续）；下方统计仍为全部技能。</span>
      </div>
      <div className={styles.tableScroll}>
      <table className={styles.table}>
        <thead>
          <tr>
            <th>技能</th>
            <th title="破防系数">Y</th>
            <th title="基础攻击">B</th>
            <th title="技能基础">I</th>
            <th title="普通命中">N</th>
            <th title="会心伤害">H</th>
            <th title="期望伤害">Q</th>
            {baseline && <th>ΔQ / Δ%</th>}
            <th title="各技能期望在当前技能池中的占比">技能池占比</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 && (
            <tr><td className={styles.emptyRow} colSpan={baseline ? 9 : 8}>没有匹配的技能，请修改搜索或重置筛选。</td></tr>
          )}
          {rows.map((row, i) => {
            const jumps = row.dot_jumps ?? [];
            const isDotRow = row.dotIndex !== null;
            const q = isDotRow ? jumps[row.dotIndex!] : row.q;
            const base = baselineByName.get(row.skill_name)?.[row.occurrence];
            const baseQ = base && (isDotRow ? base.dot_jumps?.[row.dotIndex!] : base.q);
            const difference = baseQ === undefined ? null : q - baseQ;
            const old = previous?.filter((r) => r.skill_name === row.skill_name)[row.occurrence];
            const oldQ = old && (isDotRow ? old.dot_jumps?.[row.dotIndex!] : old.q);
            const share = totalQ > 0 ? q / totalQ * 100 : 0;
            const isBest = !isDotRow && row.q === maxQ;
            const isFixed = row.has_critical_strike || (row.zhenshishanghai ?? 0) > 0;
            return (
              <tr
                key={`${row.skill_name}-${row.occurrence}-${row.dotIndex ?? "row"}`}
                className={clsx(isBest && styles.highlightRow, oldQ !== undefined && oldQ !== q && styles.changedRow)}
              >
                <td>
                  {row.skill_name}
                  {row.has_critical_strike && (
                    <span className={styles.wuzhiTag}>无质</span>
                  )}
                  {(row.zhenshishanghai ?? 0) > 0 && (
                    <span className={styles.wuzhiTag}>真实</span>
                  )}
                  {isDotRow && (
                    <span className={styles.dotTag}>DOT{row.dotIndex! + 1}</span>
                  )}
                </td>
                <td className={styles.colNum}>{formatNumber(row.y)}</td>
                <td className={styles.colNum}>{formatNumber(row.b)}</td>
                <td className={styles.colNum}>{formatNumber(row.i)}</td>
                {isFixed ? (
                  <>
                    <td className={styles.colNum}>-</td>
                    <td className={styles.colNum}>-</td>
                  </>
                ) : (
                  <>
                    <td className={styles.colNum}>{formatNumber(row.n)}</td>
                    <td className={styles.colNum}>{formatNumber(row.h)}</td>
                  </>
                )}
                <td className={styles.colNum}>{formatNumber(q)}</td>
                {baseline && <td className={clsx(styles.colNum, difference !== null && difference > 0 && styles.positive, difference !== null && difference < 0 && styles.negative)}>
                  {difference === null ? "—" : `${difference >= 0 ? "+" : ""}${formatNumber(difference)} / ${baseQ === 0 ? "—" : `${difference >= 0 ? "+" : ""}${(difference / baseQ! * 100).toFixed(1)}%`}`}
                </td>}
                <td className={styles.shareCell}><span className={styles.shareTrack}><span style={{ width: `${share}%` }} /></span>{share.toFixed(1)}%</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      </div>
      <div className={styles.stats}>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>最大期望</span>
          <span className={styles.statValue}><AnimatedNumber value={maxQ} /></span>
        </div>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>平均期望</span>
          <span className={styles.statValue}><AnimatedNumber value={avgQ} /></span>
        </div>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>会心占比</span>
          <span className={styles.statValue}>{(critRatio * 100).toFixed(0)}%</span>
        </div>
      </div>
    </div>
  );
}

export default memo(ResultTable);
