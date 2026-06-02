import { useState, useEffect } from "react";
import type { ForumFileInfo } from "../types";
import { FORUM_URL } from "../utils/constants";
import * as api from "../api/commands";
import styles from "./ForumPage.module.css";

const PAGE_SIZE = 10;

export default function ForumPage() {
  const [category, setCategory] = useState("shuxing");
  const [categories, setCategories] = useState<string[]>([]);
  const [files, setFiles] = useState<ForumFileInfo[]>([]);
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api.forumListCategories(FORUM_URL).then(setCategories).catch(() => {});
  }, []);

  useEffect(() => {
    setLoading(true);
    setPage(0);
    api.forumListFiles(FORUM_URL, category).then(setFiles).catch(() => {}).finally(() => setLoading(false));
  }, [category]);

  const filtered = files.filter((f) =>
    f.name.toLowerCase().includes(search.toLowerCase()),
  );
  const totalPages = Math.ceil(filtered.length / PAGE_SIZE);
  const pageFiles = filtered.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2>论坛数据</h2>
      </div>
      {categories.length > 1 && (
        <div className={styles.tabs}>
          {categories.map((c) => (
            <button key={c} className={c === category ? styles.tabActive : styles.tab}
              onClick={() => setCategory(c)}>{c}</button>
          ))}
        </div>
      )}
      <input className={styles.search} placeholder="搜索文件..."
        value={search} onChange={(e) => { setSearch(e.target.value); setPage(0); }} />
      {loading ? (
        <div className={styles.emptyHint}>加载中...</div>
      ) : (
        <>
          <table className={styles.fileTable}>
            <thead><tr><th>文件名</th><th>大小</th><th></th></tr></thead>
            <tbody>
              {pageFiles.map((f) => (
                <tr key={f.name}>
                  <td>{f.name}</td>
                  <td>{f.size > 1024 ? `${(f.size / 1024).toFixed(0)}KB` : `${f.size}B`}</td>
                  <td><button className={styles.downloadBtn} onClick={async () => { try { await api.forumDownloadFile(f.name, FORUM_URL, category); } catch {} }}>下载</button></td>
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
  );
}