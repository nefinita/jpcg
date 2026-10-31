import type { TimelineEventDTO } from "../types";

/**
 * 时间轴事件 → 列表文案。
 *
 * 注意：Rust 侧 `TimelineEventKindDTO` 是 `#[serde(tag = "kind", rename_all = "snake_case")]`，
 * 真实 JSON 是 { kind: "cast", skill_name, damage, hit_time } 这种**内部标签**形状，
 * 不是 { cast: { ... } } 的外部标签形状。
 */
export function formatTimelineEvent(e: TimelineEventDTO): string {
  const k = e.kind;
  switch (k.kind) {
    case "cast":
      return `施放 ${k.skill_name} @${k.hit_time.toFixed(2)}s 伤害 ${k.damage}`;
    case "dot_tick":
      return `dot ${k.skill_name} #${k.index} 伤害 ${k.damage}`;
    case "form_switch":
      return `切形态 ${k.from} → ${k.to}`;
    case "buff_gain":
      return `buff ${k.buff_id} ×${k.stacks}`;
    default: {
      const never: never = k;
      return String(never);
    }
  }
}
