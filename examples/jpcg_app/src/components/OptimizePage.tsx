import { useState, useCallback } from "react";
import type { FormData, DerivativesOutputDTO } from "../types";
import * as api from "../api/commands";
import { IconClose } from "./icons";
import styles from "./OptimizePage.module.css";

interface Props {
  formData?: FormData | null;
}

export default function OptimizePage({ formData }: Props) {
  const [output, setOutput] = useState<DerivativesOutputDTO | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedSkill, setExpandedSkill] = useState<string | null>(null);
  const [chartAttr, setChartAttr] = useState<string | null>(null);

  const handleCalculate = useCallback(async () => {
    if (!formData) return;
    setError(null);
    setLoading(true);
    try {
      const req = {
        player: {
          jcsx: formData.xinfa_config.xinfa_nom,
          jichu_shuxing: formData.player.jichu_shuxing ?? 0,
          jichu_gongji: formData.player.jichu_gongji ?? 0,
          huixin_dengji: formData.player.huixin_dengji ?? 0,
          huixin_xiaoguo: formData.player.huixin_xiaoguo ?? 0,
          pofang_dengji: formData.player.pofang_dengji ?? 0,
          wuqi_shanghai: formData.player.wuqi_shanghai ?? 0,
        },
        hostile: {
          waigong_fangyu: formData.hostile.waigong_fangyu ?? 0,
          neigong_fangyu: formData.hostile.neigong_fangyu ?? 0,
          yujin_dengji: formData.hostile.yujin_dengji ?? 0,
          huajin_dengji: formData.hostile.huajin_dengji ?? 0,
          jianshang_bili: formData.hostile.jianshang_bili ?? 0,
          target_hp: formData.hostile.target_hp ?? 0,
        },
        xinfa_config: formData.xinfa_config,
        buff: formData.buff,
        coefficient: formData.coefficient,
      };
      const data = await api.computeDerivatives(req);
      setOutput(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [formData]);

  if (!formData) {
    return (
      <div className={styles.page}>
        <div className={styles.empty}>请先在"计算"页面填入属性并完成一次计算</div>
      </div>
    );
  }

  const chartSkill =
    chartAttr && output
      ? output.derivatives.find((d) => d.attr_id === chartAttr)
      : null;

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2>导数分析</h2>
      </div>

      <div className={styles.controls}>
        <button className={styles.primaryBtn} onClick={handleCalculate} disabled={loading}>
          {loading ? "计算中..." : "计算导数"}
        </button>
      </div>

      {error && <div className={styles.error}>{error}</div>}

      {output && (
        <>
          {/* 推荐区 */}
          <section className={styles.section}>
            <div className={styles.sectionTitle}>优化方向</div>
            <div className={styles.recommendCard}>
              <div className={styles.recommendRow}>
                会心 vs 破防：
                <strong className={styles.better}>{output.recommendation.crit_vs_pofang.better}</strong>
                <span className={styles.muted}>（会心 {output.recommendation.crit_vs_pofang.huixin_total.toFixed(1)} · 破防 {output.recommendation.crit_vs_pofang.pofang_total.toFixed(1)}）</span>
              </div>
              <div className={styles.recommendRow}>
                Top 3 推荐：
                {output.recommendation.top3.map((t, i) => (
                  <span key={t.attr_id} className={styles.topChip}>
                    #{i + 1} {t.attr_name} ({t.total_derivative.toFixed(1)})
                  </span>
                ))}
              </div>
            </div>
          </section>

          {/* 导数排行表 */}
          <section className={styles.section}>
            <div className={styles.sectionTitle}>各属性导数排行</div>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>#</th>
                  <th>属性</th>
                  <th>当前值</th>
                  <th>总导数 (∑ΔQ)</th>
                  <th>各技能展开</th>
                </tr>
              </thead>
              <tbody>
                {output.derivatives.map((d, idx) => (
                  <>
                    <tr key={d.attr_id}>
                      <td>{idx + 1}</td>
                      <td className={styles.attrName}>{d.attr_name}</td>
                      <td>{d.current_value.toFixed(0)}</td>
                      <td className={styles.benefitCell}>{d.total_derivative.toFixed(4)}</td>
                      <td>
                        <button
                          className={styles.expandBtn}
                          onClick={() => setExpandedSkill(expandedSkill === d.attr_id ? null : d.attr_id)}
                        >
                          {expandedSkill === d.attr_id ? "收起" : "展开"}
                        </button>
                      </td>
                    </tr>
                    {expandedSkill === d.attr_id && (
                      <tr key={`${d.attr_id}-skills`}>
                        <td colSpan={5}>
                          <div className={styles.skillDerivList}>
                            {d.per_skill.map((s) => (
                              <div
                                key={s.skill_name}
                                className={styles.skillDerivItem}
                                onClick={() => setChartAttr(chartAttr === d.attr_id ? null : d.attr_id)}
                              >
                                <span className={styles.skillName}>{s.skill_name}</span>
                                <span className={styles.skillDeriv}>{s.derivative > 0 ? "+" : ""}{s.derivative.toFixed(4)}</span>
                              </div>
                            ))}
                          </div>
                        </td>
                      </tr>
                    )}
                  </>
                ))}
              </tbody>
            </table>
          </section>

          {/* 单属性柱状图 */}
          {chartSkill && (
            <section className={styles.section}>
              <div className={styles.sectionTitle}>
                {chartSkill.attr_name} — 各技能导数对比
                <button className={styles.closeBtn} onClick={() => setChartAttr(null)} aria-label="关闭"><IconClose size={15} /></button>
              </div>
              <div className={styles.barChart}>
                {chartSkill.per_skill.map((s) => {
                  const maxDer = Math.max(...chartSkill.per_skill.map((ps) => Math.abs(ps.derivative)), 1);
                  const pct = (s.derivative / maxDer) * 100;
                  const positive = s.derivative >= 0;
                  return (
                    <div key={s.skill_name} className={styles.barRow}>
                      <span className={styles.barLabel}>{s.skill_name}</span>
                      <div className={styles.barTrack}>
                        <div
                          className={`${styles.barFill} ${positive ? styles.barPositive : styles.barNegative}`}
                          style={{ width: `${Math.abs(pct)}%` }}
                        />
                      </div>
                      <span className={styles.barValue}>{s.derivative > 0 ? "+" : ""}{s.derivative.toFixed(2)}</span>
                    </div>
                  );
                })}
              </div>
            </section>
          )}
        </>
      )}
    </div>
  );
}
