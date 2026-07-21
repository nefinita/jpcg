import { useState } from "react";
import {
  Calculator,
  ChevronLeft,
  ChevronRight,
  Globe2,
  SlidersHorizontal,
  Waypoints,
  type LucideIcon,
} from "lucide-react";
import clsx from "../utils/clsx";
import styles from "./ActivityBar.module.css";

export type AppPage = "calc" | "forum" | "combo" | "attribute";

interface Props {
  currentPage: AppPage;
  onNavigate: (page: AppPage) => void;
}

const ITEMS: ReadonlyArray<{ page: AppPage; icon: LucideIcon; label: string }> = [
  { page: "calc", icon: Calculator, label: "计算" },
  { page: "forum", icon: Globe2, label: "论坛" },
  { page: "combo", icon: Waypoints, label: "排轴器" },
  { page: "attribute", icon: SlidersHorizontal, label: "属性配置" },
];

export default function ActivityBar({ currentPage, onNavigate }: Props) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className={clsx(styles.bar, expanded && styles.expanded)}>
      {ITEMS.map(({ page, icon: Icon, label }) => (
        <button
          key={page}
          className={clsx(styles.btn, currentPage === page && styles.active)}
          onClick={() => onNavigate(page)}
          title={label}
        >
          <span className={styles.icon}><Icon size={18} strokeWidth={1.8} /></span>
          {expanded && <span className={styles.label}>{label}</span>}
        </button>
      ))}
      <button
        className={styles.toggleBtn}
        onClick={() => setExpanded((v) => !v)}
        title={expanded ? "收起" : "展开"}
      >
        {expanded ? <ChevronLeft size={15} /> : <ChevronRight size={15} />}
      </button>
    </div>
  );
}
