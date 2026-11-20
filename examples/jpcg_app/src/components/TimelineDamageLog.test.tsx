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
    { time: 0, cumulative: 10_000, kind: { kind: "cast", skill_name: "宫", damage: 10_000, hit_time: 0, gcd_secs: 1.5, gcd_ready_at: 1.5 } },
    { time: 1.5, cumulative: 22_000, kind: { kind: "cast", skill_name: "商", damage: 12_000, hit_time: 1.5, gcd_secs: 1.5, gcd_ready_at: 3 } },
    { time: 3, cumulative: 30_000, kind: { kind: "dot_tick", skill_name: "商", index: 1, damage: 8_000 } },
    { time: 3, cumulative: 30_000, kind: { kind: "form_switch", from: "qu-yangchun", to: "jian" } },
    { time: 4, cumulative: 30_000, kind: { kind: "buff_gain", buff_id: "gaoshan", stacks: 1 } },
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
  expect(formatTime(0)).toBe("0:00.0");
  expect(formatTime(63.9)).toBe("1:03.9");
  act(() => root.render(<TimelineDamageLog events={result.events} />));
  const text = container.textContent ?? "";
  expect(text).toContain("0:00.0");
  expect(text).toContain("10,000");
  expect(text).toContain("22,000");
  expect(text).toContain("30,000");
  expect(text).toContain("GCD 1.50s（就绪 1.50s）");
  expect(text).toContain("第 1 跳");
  expect(text).toContain("qu-yangchun → jian");
});

it("器形与形态切换使用后端职业数据提供的显示名", () => {
  const events: TimelineResultDTO["events"] = [
    { time: 0, kind: { kind: "form_switch", from: "single", to: "dual" } },
    { time: 0, kind: { kind: "qifeng_switch", from: "feng", to: "lei" } },
  ];
  const names: Record<string, string> = { single: "单持", dual: "双持", feng: "风", lei: "雷" };
  act(() => root.render(<TimelineDamageLog events={events} formName={(id) => names[id] ?? id} qifengName={(id) => names[id] ?? id} />));
  expect(container.textContent).toContain("单持 → 双持");
  expect(container.textContent).toContain("风 → 雷");
});

it("保留后端按动作归属筛选的边界读条命中，并直接展示后端累计", () => {
  const events = [{ time: 2, cumulative: 12345, kind: { kind: "cast" as const, skill_name: "边界读条", damage: 12345, hit_time: 2 } }];
  act(() => root.render(<TimelineDamageLog events={events} killStart={0} killEnd={1.999} />));
  expect(container.querySelectorAll("tbody tr")).toHaveLength(1);
  expect(container.textContent).toContain("边界读条");
  expect(container.querySelector("tbody tr td:last-child")?.textContent).toBe("12,345");
});
