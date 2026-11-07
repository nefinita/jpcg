import type { TimelineEventDTO } from "../types";
import styles from "./TimelineDamageLog.module.css";

const pad = (n: number) => String(Math.floor(n)).padStart(2, "0");
/** 分:秒.十分位（0.1s 精度，便于区分「施放」和 0.5s 后的 DOT 首跳） */
export function formatTime(seconds: number): string {
  const total = Math.max(0, seconds);
  const minutes = Math.floor(total / 60);
  const rest = total - minutes * 60;
  return `${minutes}:${rest.toFixed(1).padStart(4, "0")}`;
}

const kindLabel = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return "施放";
    case "dot_tick": return "持续";
    case "channel_tick": return "引导";
    case "form_switch": return "器形";
    case "qifeng_switch": return "曲风";
    case "buff_gain": return "增益";
  }
};

const kindClass = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return styles.kindCast;
    case "dot_tick": return styles.kindDot;
    case "channel_tick": return styles.kindCast;
    case "form_switch":
    case "qifeng_switch": return styles.kindSwitch;
    default: return "";
  }
};

const kindDetail = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast":
      return kind.gcd_secs != null && kind.gcd_secs > 0
        ? `GCD ${kind.gcd_secs.toFixed(2)}s（就绪 ${(kind.gcd_ready_at ?? 0).toFixed(2)}s）`
        : "不占 GCD";
    case "dot_tick": return `第 ${kind.index} 跳${kind.advanced ? "（提前）" : ""}`;
    case "channel_tick": return `第 ${kind.index} 段`;
    case "form_switch": return `${kind.from} → ${kind.to}`;
    case "qifeng_switch": return `${kind.from} → ${kind.to}`;
    case "buff_gain": return `${kind.buff_id} ×${kind.stacks}`;
  }
};

const kindDamage = (kind: TimelineEventDTO["kind"]) =>
  kind.kind === "cast" || kind.kind === "dot_tick" || kind.kind === "channel_tick" ? Math.round(kind.damage) : 0;

const kindName = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return kind.skill_name;
    case "dot_tick": return kind.skill_name;
    case "channel_tick": return kind.skill_name;
    case "form_switch": return "器形切换";
    case "qifeng_switch": return "曲风切换";
    case "buff_gain": return "增益";
  }
};

export function TimelineDamageLog({ events }: { events: TimelineEventDTO[] }) {
  // 累计伤害由后端逐事件给出，前端不再自己求和
  const rows = events.map((event, index) => ({
    event,
    index,
    damage: kindDamage(event.kind),
    cumulative: event.cumulative ?? 0,
  }));
  return (
    <div className={styles.root}>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <colgroup>
            <col style={{ width: 24 }} />
            <col style={{ width: 44 }} />
            <col style={{ width: 46 }} />
            <col />
            <col style={{ width: 58 }} />
            <col style={{ width: 58 }} />
          </colgroup>
          <thead>
            <tr>
              <th>#</th>
              <th>时间</th>
              <th>类型</th>
              <th>技能</th>
              <th className={styles.right}>伤害</th>
              <th className={styles.right}>累计</th>
            </tr>
          </thead>
          <tbody>
            {rows.map(({ event, index, damage, cumulative: sum }) => (
              <tr key={index}>
                <td className={styles.idx}>{index + 1}</td>
                <td className={styles.time}>{formatTime(event.time)}</td>
                <td>
                  <span className={`${styles.kind} ${kindClass(event.kind)}`}>{kindLabel(event.kind)}</span>
                </td>
                <td>
                  <div className={styles.skillName}>{kindName(event.kind)}</div>
                  <div className={styles.skillDetail}>{kindDetail(event.kind)}</div>
                </td>
                <td className={styles.num}>{damage > 0 ? damage.toLocaleString() : "—"}</td>
                <td className={styles.cum}>{sum.toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
