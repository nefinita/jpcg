import clsx from "../utils/clsx";
import styles from "./ActivityBar.module.css";

interface Props {
  activePanel: "forum" | "combo" | null;
  onToggle: (panel: "forum" | "combo") => void;
}

export default function ActivityBar({ activePanel, onToggle }: Props) {
  return (
    <div className={styles.bar}>
      <button
        className={clsx(styles.btn, activePanel === "forum" && styles.active)}
        onClick={() => onToggle("forum")}
        title="论坛"
      >
        🌐
      </button>
      <button
        className={clsx(styles.btn, activePanel === "combo" && styles.active)}
        onClick={() => onToggle("combo")}
        title="连招"
      >
        🔗
      </button>
    </div>
  );
}
