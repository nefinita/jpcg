import { describe, expect, it } from "vitest";
import { formatTimelineEvent } from "./timelineFormat";
import type { TimelineEventDTO } from "../types";

// 与 Rust `TimelineEventDTO` 序列化结果一致的样本（内部标签，snake_case）
const RUST_SHAPED_EVENTS: TimelineEventDTO[] = [
  { time: 0, kind: { kind: "cast", skill_name: "断云势", damage: 12345, hit_time: 0 } },
  { time: 1.5, kind: { kind: "dot_tick", skill_name: "疏曲", index: 1, damage: 678 } },
  { time: 1.5, kind: { kind: "form_switch", from: "single", to: "dual" } },
  { time: 1.5, kind: { kind: "buff_gain", buff_id: "burst", stacks: 2 } },
];

describe("formatTimelineEvent", () => {
  it("格式化 Rust 内部标签形状的四种事件", () => {
    // 经 JSON 往返，确保形状就是真实响应形状（而非外部标签）
    const events = JSON.parse(JSON.stringify(RUST_SHAPED_EVENTS)) as TimelineEventDTO[];
    expect(formatTimelineEvent(events[0])).toBe("施放 断云势 @0.00s 伤害 12345");
    expect(formatTimelineEvent(events[1])).toBe("dot 疏曲 #1 伤害 678");
    expect(formatTimelineEvent(events[2])).toBe("切形态 single → dual");
    expect(formatTimelineEvent(events[3])).toBe("buff burst ×2");
  });
});
