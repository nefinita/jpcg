import styles from "./StatusBar.module.css";

interface Props {
  message: string;
}

export default function StatusBar({ message }: Props) {
  return (
    <div className={styles.bar}>
      <span className={styles.dot} />
      <span>{message}</span>
    </div>
  );
}
