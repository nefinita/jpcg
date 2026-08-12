import type { Toast as ToastType } from "../hooks/useToast";
import { IconCheck, IconClose, IconAlert } from "./icons";
import styles from "./Toast.module.css";

interface Props {
  toasts: ToastType[];
  onRemove: (id: number) => void;
}

const ICONS: Record<string, React.ReactNode> = {
  success: <IconCheck size={15} />,
  error: <IconClose size={15} />,
  warning: <IconAlert size={15} />,
  info: <IconAlert size={15} />,
};

export default function Toast({ toasts, onRemove }: Props) {
  if (toasts.length === 0) return null;
  return (
    <div className={styles.container}>
      {toasts.map((t) => (
        <div key={t.id} className={`${styles.toast} ${styles[t.type]}`}>
          <span className={styles.icon}>{ICONS[t.type]}</span>
          <span>{t.message}</span>
          <button className={styles.close} onClick={() => onRemove(t.id)}>
            <IconClose size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
