import type { SkillResultDTO } from "../types";
import { formatNumber } from "../utils/format";
import clsx from "../utils/clsx";
import styles from "./ResultTable.module.css";

interface Props {
  results: SkillResultDTO[] | null;
  calculating: boolean;
}

export default function ResultTable({ results, calculating }: Props) {
  if (calculating) {
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

  const maxQ = Math.max(...results.map((r) => r.q));
  const avgQ = Math.round(results.reduce((s, r) => s + r.q, 0) / results.length);
  const critSkills = results.filter((r) => r.h > r.n);
  const critRatio = results.length > 0 ? critSkills.length / results.length : 0;

  // DOT 技能展开为每跳一行（dotIndex = 跳序 0..n-1）；非 DOT 技能保持原行
  const rows = results.flatMap((r) => {
    const jumps = r.dot_jumps ?? [];
    if (jumps.length === 0) return [{ ...r, dotIndex: null as number | null }];
    return jumps.map((_, k) => ({ ...r, dotIndex: k }));
  });

  return (
    <div className={styles.card}>
      <div className={styles.header}>
        <span>计算结果</span>
        <span style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>
          {results.length} 个技能
        </span>
      </div>
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
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => {
            const jumps = row.dot_jumps ?? [];
            const isDotRow = row.dotIndex !== null;
            const q = isDotRow ? jumps[row.dotIndex!] : row.q;
            const isBest = !isDotRow && row.q === maxQ;
            const isFixed = row.has_critical_strike || (row.zhenshishanghai ?? 0) > 0;
            return (
              <tr
                key={i}
                className={clsx(isBest && styles.highlightRow)}
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
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className={styles.stats}>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>最大期望</span>
          <span className={styles.statValue}>{formatNumber(maxQ)}</span>
        </div>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>平均期望</span>
          <span className={styles.statValue}>{formatNumber(avgQ)}</span>
        </div>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>会心占比</span>
          <span className={styles.statValue}>{(critRatio * 100).toFixed(0)}%</span>
        </div>
      </div>
    </div>
  );
}
