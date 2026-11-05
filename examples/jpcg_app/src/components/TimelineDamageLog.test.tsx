import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it } from "vitest";
import { TimelineDamageLog, formatTime } from "./TimelineDamageLog";
import type { TimelineResultDTO } from "../types";

const result: TimelineResultDTO = {
  total_damage: 30_000,
  window_dps: 10_000,
  skipped_actions: 0,
  events: [
    { time: 0, kind: { kind: "cast", skill_name: "宫", damage: 10_000, hit_time: 0, gcd_secs: 1.5, gcd_ready_at: 1.5 } },
    { time: 1.5, kind: { kind: "cast", skill_name: "商", damage: 12_000, hit_time: 1.5, gcd_secs: 1.5, gcd_ready_at: 3 } },
    { time: 3, kind: { kind: "dot_tick", skill_name: "商", index: 0, damage: 8_000 } },
    { time: 3, kind: { kind: "form_switch", from: "qu-yangchun", to: "jian" } },
    { time: 4, kind: { kind: "buff_gain", buff_id: "gaoshan", stacks: 1 } },
  ],
};

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  container = document.createElement("div");
  document.body.append(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
});

it("按分:秒展示时间，并按事件顺序累计伤害", () => {
  expect(formatTime(0)).toBe("0:00");
  expect(formatTime(63.9)).toBe("1:03");
  act(() => root.render(<TimelineDamageLog events={result.events} />));
  const text = container.textContent ?? "";
  expect(text).toContain("0:00");
  expect(text).toContain("10,000");
  expect(text).toContain("22,000");
  expect(text).toContain("30,000");
  expect(text).toContain("GCD 1.50s（就绪 1.50s）");
  expect(text).toContain("第 1 跳");
  expect(text).toContain("qu-yangchun → jian");
});
