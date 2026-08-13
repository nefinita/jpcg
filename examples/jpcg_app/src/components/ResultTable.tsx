import type { SkillResultDTO } from "../types";
import { formatNumber } from "../utils/format";
import clsx from "../utils/clsx";
import styles from "./ResultTable.module.css";

interface Props {
  results: SkillResultDTO[] | null;
  calculating: boolean;
}

function isDot(name: string): boolean {
  return name.includes("（dot）") || name.includes("(dot)") || name.includes("dot");
}

function isUltimate(name: string): boolean {
  return name.includes("相依");
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
          {results.map((r, i) => {
            const isBest = r.q === maxQ;
            const dot = isDot(r.skill_name) || (r.dot_jumps?.length ?? 0) > 0;
            return (
              <tr
                key={i}
                className={clsx(isBest && styles.highlightRow)}
              >
                <td>
                  {r.skill_name}
                  {dot && <span className={styles.dotTag}>DOT</span>}
                </td>
                <td className={styles.colNum}>{formatNumber(r.y)}</td>
                <td className={styles.colNum}>{formatNumber(r.b)}</td>
                <td className={styles.colNum}>{formatNumber(r.i)}</td>
                <td className={styles.colNum}>{formatNumber(r.n)}</td>
                <td className={styles.colNum}>{formatNumber(r.h)}</td>
                <td className={styles.colNum}>
                  {formatNumber(r.q)}
                  {r.dot_jumps?.length > 0 && (
                    <div className={styles.dotJumps}>
                      {r.dot_jumps.map((j, k) => (
                        <span key={k} title={`第${k + 1}跳`}>{formatNumber(j)}</span>
                      ))}
                    </div>
                  )}
                </td>
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
