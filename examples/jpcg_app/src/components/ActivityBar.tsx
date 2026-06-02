import { useState } from "react";
import clsx from "../utils/clsx";
import styles from "./ActivityBar.module.css";

interface Props {
  currentPage: "calc" | "forum" | "combo";
  onNavigate: (page: "calc" | "forum" | "combo") => void;
}

const ITEMS = [
  { page: "calc" as const, icon: "📊", label: "计算" },
  { page: "forum" as const, icon: "🌐", label: "论坛" },
  { page: "combo" as const, icon: "🔗", label: "排轴器" },
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
