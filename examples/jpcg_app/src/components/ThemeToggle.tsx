import { IconSun, IconMoon } from "./icons";
import styles from "./ThemeToggle.module.css";

interface Props {
  theme: "dark" | "light";
  onToggle: () => void;
}

export default function ThemeToggle({ theme, onToggle }: Props) {
  return (
    <button className={styles.btn} onClick={onToggle} title="切换主题">
      {theme === "dark" ? <IconSun size={16} /> : <IconMoon size={16} />}
    </button>
  );
}
