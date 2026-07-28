import { useState } from "react";
import clsx from "../utils/clsx";
import styles from "./ActivityBar.module.css";

export type Page = "calc" | "forum" | "combo" | "editor" | "optimize";

interface Props {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const ITEMS = [
  { page: "calc" as const, icon: "📊", label: "计算" },
  { page: "forum" as const, icon: "🌐", label: "论坛" },
  { page: "combo" as const, icon: "🔗", label: "排轴器" },
  { page: "optimize" as const, icon: "📈", label: "加点优化" },
  { page: "editor" as const, icon: "✏️", label: "技能编辑" },
];

export default function ActivityBar({ currentPage, onNavigate }: Props) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className={clsx(styles.bar, expanded && styles.expanded)}>
      {ITEMS.map(({ page, icon, label }) => (
        <button
          key={page}
          className={clsx(styles.btn, currentPage === page && styles.active)}
          onClick={() => onNavigate(page)}
          title={label}
        >
          <span className={styles.icon}>{icon}</span>
          {expanded && <span className={styles.label}>{label}</span>}
        </button>
      ))}
      <button
        className={styles.toggleBtn}
        onClick={() => setExpanded((v) => !v)}
        title={expanded ? "收起" : "展开"}
      >
        {expanded ? "◀" : "▶"}
      </button>
    </div>
  );
}
