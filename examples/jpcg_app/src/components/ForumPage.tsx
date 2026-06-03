import { useState, useEffect, useCallback } from "react";
import type { ForumFileInfo } from "../types";
import { FORUM_URL } from "../utils/constants";
import * as api from "../api/commands";
import styles from "./ForumPage.module.css";

const PAGE_SIZE = 10;

interface Props {
  addToast?: (msg: string, type?: "success" | "error" | "warning" | "info") => void;
}

export default function ForumPage({ addToast }: Props) {
  const [category, setCategory] = useState("shuxing");
  const [categories, setCategories] = useState<string[]>([]);
  const [files, setFiles] = useState<ForumFileInfo[]>([]);
  const [downloadedFiles, setDownloadedFiles] = useState<string[]>([]);
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api.forumListCategories(FORUM_URL).then(setCategories).catch(() => {});
  }, []);

  const refreshDownloaded = useCallback(() => {
    api.forumListDownloaded(category).then(setDownloadedFiles).catch(() => {});
  }, [category]);

  useEffect(() => {
    setLoading(true);
    setPage(0);
    Promise.all([
      api.forumListFiles(FORUM_URL, category),
      api.forumListDownloaded(category).catch(() => [] as string[]),
    ]).then(([fileList, downloaded]) => {
      setFiles(fileList);
      setDownloadedFiles(downloaded);
    }).catch(() => {}).finally(() => setLoading(false));
  }, [category]);

  const handleDownload = useCallback(async (filename: string) => {
    try {
      await api.forumDownloadFile(filename, FORUM_URL, category);
      addToast?.("已下载: " + filename, "success");
      refreshDownloaded();
    } catch (err) {
      addToast?.(String(err), "error");
    }
  }, [category, addToast, refreshDownloaded]);

  const handleDelete = useCallback(async (filename: string) => {
    if (!window.confirm(`确定删除 ${filename}？`)) return;
    try {
      await api.forumDeleteDownloaded(filename, category);
      addToast?.("已删除: " + filename, "info");
      refreshDownloaded();
    } catch (err) {
      addToast?.(String(err), "error");
    }
  }, [category, addToast, refreshDownloaded]);

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
              {pageFiles.map((f) => {
                const isDownloaded = downloadedFiles.includes(f.name);
                return (
                  <tr key={f.name}>
                    <td>{f.name}</td>
                    <td>{f.size > 1024 ? `${(f.size / 1024).toFixed(0)}KB` : `${f.size}B`}</td>
                    <td className={styles.actionCell}>
                      {isDownloaded ? (
                        <>
                          <span className={styles.downloadedBadge}>已下载</span>
                          <button className={styles.deleteBtn} onClick={() => handleDelete(f.name)}>删除</button>
                        </>
                      ) : (
                        <button className={styles.downloadBtn} onClick={() => handleDownload(f.name)}>下载</button>
                      )}
                    </td>
                  </tr>
                );
              })}
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