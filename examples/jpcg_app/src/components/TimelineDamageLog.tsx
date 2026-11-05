import type { TimelineEventDTO } from "../types";

const pad = (n: number) => String(Math.floor(n)).padStart(2, "0");
export function formatTime(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  return `${Math.floor(total / 60)}:${pad(total % 60)}`;
}

const kindLabel = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return "施放";
    case "dot_tick": return "持续伤害";
    case "form_switch": return "切形态";
    case "buff_gain": return "获得效果";
  }
};

const kindDetail = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return kind.gcd_secs != null && kind.gcd_secs > 0 ? `GCD ${kind.gcd_secs.toFixed(2)}s（就绪 ${(kind.gcd_ready_at ?? 0).toFixed(2)}s）` : "不占 GCD";
    case "dot_tick": return `第 ${kind.index + 1} 跳`;
    case "form_switch": return `${kind.from} → ${kind.to}`;
    case "buff_gain": return `${kind.buff_id} ×${kind.stacks}`;
  }
};

const kindDamage = (kind: TimelineEventDTO["kind"]) => (kind.kind === "cast" || kind.kind === "dot_tick" ? Math.round(kind.damage) : 0);

const kindName = (kind: TimelineEventDTO["kind"]) => {
  switch (kind.kind) {
    case "cast": return kind.skill_name;
    case "dot_tick": return kind.skill_name;
    case "form_switch": return "形态切换";
    case "buff_gain": return "增益";
  }
};

export function TimelineDamageLog({ events }: { events: TimelineEventDTO[] }) {
  let cumulative = 0;
  const rows = events.map((event, index) => {
    const damage = kindDamage(event.kind);
    cumulative += damage;
    return { event, index, damage, cumulative };
  });
  return <div style={{ maxHeight: 260, overflowY: "auto", fontSize: 12 }}>
    <table style={{ width: "100%", borderCollapse: "collapse" }}>
      <thead style={{ position: "sticky", top: 0, background: "var(--glass-bg, transparent)" }}>
        <tr style={{ textAlign: "left", opacity: 0.75 }}>
          <th style={{ padding: "2px 6px" }}>#</th>
          <th style={{ padding: "2px 6px" }}>时间</th>
          <th style={{ padding: "2px 6px" }}>类型</th>
          <th style={{ padding: "2px 6px" }}>技能</th>
          <th style={{ padding: "2px 6px", textAlign: "right" }}>伤害</th>
          <th style={{ padding: "2px 6px", textAlign: "right" }}>累计</th>
          <th style={{ padding: "2px 6px" }}>备注</th>
        </tr>
      </thead>
      <tbody>
        {rows.map(({ event, index, damage, cumulative: sum }) => (
          <tr key={index} style={{ borderTop: "1px solid var(--border-subtle, rgba(255,255,255,.06))" }}>
            <td style={{ padding: "2px 6px", opacity: 0.5 }}>{index + 1}</td>
            <td style={{ padding: "2px 6px", fontVariantNumeric: "tabular-nums" }}>{formatTime(event.time)}</td>
            <td style={{ padding: "2px 6px" }}>{kindLabel(event.kind)}</td>
            <td style={{ padding: "2px 6px" }}>{kindName(event.kind)}</td>
            <td style={{ padding: "2px 6px", textAlign: "right", fontVariantNumeric: "tabular-nums" }}>{damage > 0 ? damage.toLocaleString() : "—"}</td>
            <td style={{ padding: "2px 6px", textAlign: "right", fontVariantNumeric: "tabular-nums", opacity: 0.8 }}>{sum.toLocaleString()}</td>
            <td style={{ padding: "2px 6px", opacity: 0.6 }}>{kindDetail(event.kind)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  </div>;
}
