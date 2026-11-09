import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import ResultTable from "./ResultTable";
import type { SkillResultDTO } from "../types";

const skill = (skill_name: string, q: number, extra: Partial<SkillResultDTO> = {}): SkillResultDTO => ({
  skill_name, q, y: 1, b: 2, i: 3, n: 4, h: 5, dot_jumps: [],
  has_critical_strike: false, zhenshishanghai: 0, lost_hp_zhenshishanghai: 0, ...extra,
});
const results = [skill("破招 Alpha", 9), skill("流血", 100, { dot_jumps: [30, 70] }), skill("破招 Beta", 20), skill("同值", 20)];
let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
  container = document.createElement("div"); document.body.appendChild(container); root = createRoot(container);
});
afterEach(async () => { await act(async () => root.unmount()); container.remove(); });
async function render(data: SkillResultDTO[] | null = results, calculating = false) {
  await act(async () => root.render(<ResultTable results={data} calculating={calculating} />));
}
async function search(value: string) {
  const input = container.querySelector("input")!;
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}
async function sort(value: string) {
  const select = container.querySelector("select")!;
  await act(async () => { select.value = value; select.dispatchEvent(new Event("change", { bubbles: true })); });
}
const names = () => Array.from(container.querySelectorAll("tbody tr")).map((tr) => tr.children[0].textContent);
const qValues = () => Array.from(container.querySelectorAll("tbody tr")).map((tr) => tr.children[6]?.textContent);
async function reset() { await act(async () => container.querySelector("button")!.click()); }

describe("计算结果技能筛选与排序", () => {
  it("默认保留原序、DOT 跳序及全部统计", async () => {
    await render(); expect(names()).toEqual(["破招 Alpha", "流血DOT1", "流血DOT2", "破招 Beta", "同值"]);
    expect(qValues()).toEqual(["9", "30", "70", "20", "20"]);
    expect(container.textContent).toContain("4 / 4 个技能");
  });
  it("按名称子串筛选，忽略首尾空白及大小写，清空恢复", async () => {
    await render(); await search(" ALPHA "); expect(names()).toEqual(["破招 Alpha"]);
    expect(container.textContent).toContain("1 / 4 个技能");
    await search("破招"); expect(names()).toEqual(["破招 Alpha", "破招 Beta"]);
    await search(""); expect(names()).toHaveLength(5);
  });
  it("数值排序不是字符串排序，同值稳定，DOT 始终成组", async () => {
    const original = JSON.stringify(results); await render(); await sort("q-desc");
    expect(names()).toEqual(["流血DOT1", "流血DOT2", "破招 Beta", "同值", "破招 Alpha"]);
    expect(qValues()).toEqual(["30", "70", "20", "20", "9"]);
    await sort("q-asc"); expect(names()).toEqual(["破招 Alpha", "破招 Beta", "同值", "流血DOT1", "流血DOT2"]);
    expect(JSON.stringify(results)).toBe(original);
  });
  it("筛选与排序组合，重置同时恢复且不改变全体统计", async () => {
    await render(); const stats = container.querySelector('[class*="stats"]')!.textContent;
    await sort("q-desc"); await search("破招"); expect(names()).toEqual(["破招 Beta", "破招 Alpha"]);
    expect(container.querySelector('[class*="stats"]')!.textContent).toBe(stats);
    await reset(); expect(names()[0]).toBe("破招 Alpha"); expect(names()).toHaveLength(5);
  });
  it("无匹配有独立提示，可以从空列表重置", async () => {
    await render(); await search("不存在"); expect(container.textContent).toContain("没有匹配的技能");
    expect(container.querySelector('[role="status"]')!.textContent).toContain("0 / 4 个技能");
    expect(container.querySelector('[role="status"]')!.getAttribute("aria-atomic")).toBe("true"); await reset(); expect(names()).toHaveLength(5);
  });
  it("名称排序及重复技能名称不会丢失结果", async () => {
    const data = [skill("乙", 1), skill("甲", 2), skill("甲", 3)]; await render(data); await sort("name");
    expect(names()).toEqual(["甲", "甲", "乙"]); expect(qValues()).toEqual(["2", "3", "1"]);
  });
  it("计算中/清空/新结果切换保留用户筛选但使用最新数据", async () => {
    await render(); await search("破招"); await sort("q-desc"); await render(results, true);
    expect(container.textContent).toContain("计算中"); await render(null); expect(container.textContent).toContain("配置属性");
    await render([skill("破招 New", 50), skill("破招 Low", 2)]);
    expect(names()).toEqual(["破招 New", "破招 Low"]); await reset(); expect(container.querySelector("input")!.value).toBe("");
    await render([]); expect(container.textContent).toContain("配置属性");
  });
  it("无质与真实伤害标签及不可用 N/H 不因排序改变", async () => {
    await render([skill("无质技", 2, { has_critical_strike: true }), skill("真伤技", 8, { zhenshishanghai: 8 })]);
    await sort("q-desc"); expect(names()).toEqual(["真伤技真实", "无质技无质"]);
    expect(Array.from(container.querySelectorAll("tbody tr")).every((tr) => tr.children[4].textContent === "-" && tr.children[5].textContent === "-")).toBe(true);
  });
});

describe("钉住基准差值", () => {
  it("仅钉住后显示；按原始重名顺序匹配、支持 DOT 单跳、零基准与缺项", async () => {
    const data = [skill("重名", 20), skill("重名", 5), skill("DOT", 30, { dot_jumps: [10, 20] }), skill("新增", 4)];
    const base = [skill("重名", 10), skill("重名", 0), skill("DOT", 20, { dot_jumps: [8, 12] })];
    await act(async () => root.render(<ResultTable results={data} calculating={false} baseline={base} />));
    const differences = () => [...container.querySelectorAll("tbody tr")].map((row) => row.children[7].textContent);
    expect(differences()).toEqual(["+10 / +100.0%", "+5 / —", "+2 / +25.0%", "+8 / +66.7%", "—"]);
    await sort("q-asc"); expect(differences()).toEqual(["—", "+5 / —", "+10 / +100.0%", "+2 / +25.0%", "+8 / +66.7%"]);
    await render(data); expect(container.textContent).not.toContain("ΔQ");
  });
  it("负差值保留负号", async () => {
    await act(async () => root.render(<ResultTable results={[skill("测试", 50)]} baseline={[skill("测试", 100)]} calculating={false} />));
    expect(container.textContent).toContain("-50 / -50.0%");
  });
});
