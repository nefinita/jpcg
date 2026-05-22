import { useState, useEffect, useCallback } from "react";
import type { SkillResultDTO, ForumFileInfo } from "../types";
import { FORUM_URL } from "../utils/constants";
import * as api from "../api/commands";
import styles from "./Sidebar.module.css";

interface Props {
  panel: "forum" | "combo" | null;
  onClose: () => void;
  results: SkillResultDTO[] | null;
}

const PAGE_SIZE = 10;

export default function Sidebar({ panel, onClose, results }: Props) {
  if (!panel) return null;

  if (panel === "forum") return <ForumPanel onClose={onClose} />;
  return <ComboPanel onClose={onClose} results={results} />;
}

function ForumPanel({ onClose }: { onClose: () => void }) {
  const [files, setFiles] = useState<ForumFileInfo[]>([]);
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    api.forumListFiles(FORUM_URL).then(setFiles).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const filtered = files.filter((f) =>
    f.name.toLowerCase().includes(search.toLowerCase()),
  );
  const totalPages = Math.ceil(filtered.length / PAGE_SIZE);
  const pageFiles = filtered.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <div className={styles.sidebar}>
      <div className={styles.panel}>
        <div className={styles.panelHeader}>
          <span>论坛数据</span>
          <button onClick={onClose}>✕</button>
        </div>
        <div className={styles.panelBody}>
          <input
            className={styles.search}
            placeholder="搜索文件..."
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(0); }}
          />
          {loading ? (
            <div style={{ color: "var(--text-muted)", textAlign: "center", padding: "2rem" }}>
              加载中...
            </div>
          ) : (
            <>
              <table className={styles.fileTable}>
                <thead>
                  <tr><th>文件名</th><th>大小</th><th></th></tr>
                </thead>
                <tbody>
                  {pageFiles.map((f) => (
                    <tr key={f.name}>
                      <td>{f.name}</td>
                      <td>{f.size > 1024 ? `${(f.size / 1024).toFixed(0)}KB` : `${f.size}B`}</td>
                      <td>
                        <button
                          className={styles.downloadBtn}
                          onClick={async () => {
                            try {
                              await api.forumDownloadFile(f.name, FORUM_URL);
                            } catch {}
                          }}
                        >
                          下载
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {totalPages > 1 && (
                <div className={styles.pagination}>
                  <button disabled={page === 0} onClick={() => setPage(page - 1)}>←</button>
                  <span>{page + 1}/{totalPages}</span>
                  <button disabled={page >= totalPages - 1} onClick={() => setPage(page + 1)}>→</button>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function ComboPanel({ onClose, results }: { onClose: () => void; results: SkillResultDTO[] | null }) {
  const [sequence, setSequence] = useState<SkillResultDTO[]>([]);

  const addToCombo = useCallback((skill: SkillResultDTO) => {
    setSequence((prev) => [...prev, skill]);
  }, []);

  const removeFromCombo = useCallback((index: number) => {
    setSequence((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearSequence = useCallback(() => {
    setSequence([]);
  }, []);

  const totalQ = sequence.reduce((s, r) => s + r.q, 0);
  const avgQ = sequence.length > 0 ? Math.round(totalQ / sequence.length) : 0;

  const skillCounts: Record<string, number> = {};
  for (const s of sequence) {
    skillCounts[s.skill_name] = (skillCounts[s.skill_name] || 0) + 1;
  }

  const maxQ = Math.max(...(results ?? []).map((r) => r.q), 0);

  return (
    <div className={styles.sidebar}>
      <div className={styles.panel}>
        <div className={styles.panelHeader}>
          <span>排轴器</span>
          <div style={{ display: "flex", gap: "var(--space-sm)" }}>
            {sequence.length > 0 && (
              <button onClick={clearSequence}>清空</button>
            )}
            <button onClick={onClose}>✕</button>
          </div>
        </div>
        <div className={styles.panelBody}>
          <div className={styles.comboSequence}>
            {sequence.map((s, i) => (
              <div
                key={i}
                className={styles.comboItem}
                onDoubleClick={() => removeFromCombo(i)}
                title="双击移除"
              >
                <span className={styles.comboIndex}>{i + 1}</span>
                <span>{s.skill_name}</span>
              </div>
            ))}
            {sequence.length === 0 && (
              <span style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>
                点击下方技能添加到序列
              </span>
            )}
          </div>

          {sequence.length > 0 && (
            <>
              <div className={styles.avgDamage}>
                总伤害: {totalQ.toLocaleString("zh-CN")} | 平均: {avgQ.toLocaleString("zh-CN")}
              </div>
              <div className={styles.skillCounts}>
                {Object.entries(skillCounts).map(([name, count]) => (
                  <span key={name} className={styles.countBadge}>
                    {name} <strong>×{count}</strong>
                  </span>
                ))}
              </div>
            </>
          )}

          <div className={styles.poolHeader}>技能池</div>
          <div className={styles.comboPool}>
            {(results ?? []).map((r, i) => (
              <button
                key={i}
                className={styles.skillChip}
                onClick={() => addToCombo(r)}
                title={`Q: ${r.q.toLocaleString("zh-CN")}${r.q === maxQ ? " ★最高" : ""}`}
              >
                {r.skill_name}
              </button>
            ))}
            {(!results || results.length === 0) && (
              <span style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>
                先计算伤害来填充技能池
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
