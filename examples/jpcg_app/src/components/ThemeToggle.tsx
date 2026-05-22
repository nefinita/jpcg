import styles from "./ThemeToggle.module.css";

interface Props {
  theme: "dark" | "light";
  onToggle: () => void;
}

export default function ThemeToggle({ theme, onToggle }: Props) {
  return (
    <button className={styles.btn} onClick={onToggle} title="切换主题">
      {theme === "dark" ? "☀️" : "🌙"}
    </button>
  );
}
