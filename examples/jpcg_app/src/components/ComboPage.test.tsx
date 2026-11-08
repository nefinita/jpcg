import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import ComboPage from "./ComboPage";
import * as api from "../api/commands";
import { defaultManualForm } from "../state/cockpit";
import { STORAGE_KEYS } from "../utils/constants";
import type { SkillPoolItemDTO } from "../types";
vi.mock("../api/commands");
const skill: SkillPoolItemDTO = { skill_name: "测试技·一", skill_id: 1, sub_id: 0, base_damage1: 10, base_damage2: 20, atk_xishu: 1, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false, lost_hp_zhenshishanghai: 0 };
const form = defaultManualForm();
let root: Root; let host: HTMLDivElement;
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true }); vi.resetAllMocks(); localStorage.clear();
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
  vi.mocked(api.loadCombatBundle).mockResolvedValue({
    skills: [skill, { ...skill, skill_name: "测试技·二", sub_id: 1 }],
    combat: {
      meta: { profession: "mowen", version: "test", note: "" },
      form: [],
      qifeng: [],
      skill_combat: [],
      buff: [],
      miji: [],
      qixue: [],
    },
  } as never);
  vi.mocked(api.listComboPresets).mockResolvedValue(["已保存"]);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
async function mount(xinfa = "mowen", revision = 0) { await act(async () => root.render(<ComboPage key={`${xinfa}-${revision}`} xinfaName={xinfa} formData={form} />)); }
async function click(text: string) {
  const button = [...host.querySelectorAll<HTMLElement>("button, [data-skill-card]")]
    .find((b) => b.textContent === text || b.title === text || b.getAttribute("data-skill-name") === text)!;
  expect(button).toBeDefined(); await act(async () => button.click());
}
async function input(el: HTMLInputElement, value: string) {
  await act(async () => { Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(el, value); el.dispatchEvent(new Event("input", { bubbles: true })); });
}
it("草稿序列/步调整/名称/搜索按心法隔离，重挂载恢复且不自动覆盖预设", async () => {
  await mount(); await click("测试技·一"); await click("调整系数");
  await input(host.querySelector('input[type="number"]')!, "123"); await click("保存调整");
  await click(" 保存"); await input(host.querySelector('input[placeholder="连招名称"]')!, "未完成草稿");
  await click("取消");
  await input(host.querySelector('input[type="search"]')!, "测试");
  const saved = JSON.parse(localStorage.getItem(`${STORAGE_KEYS.comboDraft}:mowen`)!).data;
  expect(saved.sequence[0].overrides.base_damage_override).toBe(123);
  expect(saved.presetName).toBe("未完成草稿"); expect(saved.collapsedGroups).toEqual([]);
  await mount("qixiu"); expect(host.textContent).toContain("从左侧技能目录添加技能"); await click("测试技·二");
  await mount("mowen", 1); expect(host.querySelector('input[type="search"]')!.getAttribute("value")).toBe("测试");
  expect(host.textContent).toContain("测试技·一");
  await click(" 保存"); expect((host.querySelector('input[placeholder="连招名称"]') as HTMLInputElement).value).toBe("未完成草稿");
  expect(api.saveComboPreset).not.toHaveBeenCalled(); expect(api.deleteComboPreset).not.toHaveBeenCalled();
});
it("加载预设只更新当前草稿，修改后不自动写回预设", async () => {
  vi.mocked(api.loadComboPreset).mockResolvedValue({ name: "已保存", steps: [{ skill, overrides: null }] });
  await mount();
  await act(async () => { const select = host.querySelector("select")!; select.value = "已保存"; select.dispatchEvent(new Event("change", { bubbles: true })); });
  await click("清空"); await mount("mowen", 1);
  expect(host.textContent).toContain("从左侧技能目录添加技能"); expect(host.querySelector("select")!.value).toBe("已保存");
  expect(api.saveComboPreset).not.toHaveBeenCalled();
});

it("时间轴预览下发勾选的秘籍与奇穴", async () => {
  vi.mocked(api.loadCombatBundle).mockResolvedValue({
    skills: [skill],
    combat: {
      meta: { profession: "mowen", version: "test", note: "" },
      form: [],
      qifeng: [],
      skill_combat: [{ skill_id: 1, sub_id: 0, miji_ids: ["mj_gong"] }],
      buff: [],
      miji: [{ id: "mj_gong", name: "宫·秘籍", skill_id: 1, sub_id: 0, modifiers: [{ cast_time_add: -0.125 }], effects: ["运功-0.125秒"] }],
      qixue: [{ id: "qx_dongtian", name: "洞天", slot: 1 }],
    },
  } as never);
  vi.mocked(api.calculateTimeline).mockResolvedValue({
    events: [],
    total_damage: 0,
    window_dps: 0,
    skipped_actions: 0,
  });
  await mount();
  await click("测试技·一");
  // 秘籍：技能格右上角「0/4」→ 打开配置弹窗 → 勾选秘籍 → 关闭
  await click("0/4");
  const config = host.querySelector('[data-testid="miji-config"]');
  expect(config).not.toBeNull();
  await click("运功-0.125秒");
  await click("×");
  const board = host.querySelector('[data-testid="qixue-board"]')!;
  expect(board).not.toBeNull();
  const qixueButtons = [...board.querySelectorAll("button")];
  expect(qixueButtons.length).toBe(1);
  await act(async () => {
    qixueButtons[0].click();
  });
  await act(async () => { await new Promise((r) => setTimeout(r, 600)); });
  expect(api.calculateTimeline).toHaveBeenCalledWith(
    expect.objectContaining({ miji: ["mj_gong"], qixue: ["qx_dongtian"] }),
  );
});

it("混选奇穴最多选 3 个，第 4 个无效", async () => {
  const mixed = ["秋霜", "豪情", "师襄", "井仪"].map((name, i) => ({ id: `qx_m${i}`, name, slot: 7 }));
  vi.mocked(api.loadCombatBundle).mockResolvedValue({
    skills: [skill],
    combat: {
      meta: { profession: "mowen", version: "test", note: "" },
      form: [],
      qifeng: [],
      skill_combat: [{ skill_id: 1, sub_id: 0, miji_ids: ["mj_gong"] }],
      buff: [],
      miji: [{ id: "mj_gong", name: "宫·秘籍", skill_id: 1, sub_id: 0, modifiers: [{ cast_time_add: -0.125 }], effects: ["运功-0.125秒"] }],
      qixue: mixed,
    },
  } as never);
  vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], total_damage: 0, window_dps: 0, skipped_actions: 0 });
  await mount();
  await click("测试技·一");
  for (const name of ["秋霜", "豪情", "师襄", "井仪"]) await click(name);
  await act(async () => { await new Promise((r) => setTimeout(r, 600)); });
  const calls = vi.mocked(api.calculateTimeline).mock.calls;
  const req = calls[calls.length - 1]?.[0];
  expect(req?.qixue).toHaveLength(3);
  expect(req?.qixue).not.toContain("qx_m3");
});

it("未激活奇穴的门控技能不出现在技能目录", async () => {
  const gated = { ...skill, skill_name: "洞天主动", skill_id: 99, sub_id: 0 };
  vi.mocked(api.loadSkillPool).mockResolvedValue([skill, gated]);
  vi.mocked(api.loadCombatBundle).mockResolvedValue({
    skills: [skill, gated],
    combat: {
      meta: { profession: "mowen", version: "test", note: "" },
      form: [],
      qifeng: [],
      skill_combat: [{ skill_id: 99, sub_id: 0, jihuoqixue: "qx_gate" }],
      buff: [],
      miji: [],
      qixue: [{ id: "qx_gate", name: "不愧君", slot: 6 }],
    },
  } as never);
  vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], total_damage: 0, window_dps: 0, skipped_actions: 0 });
  await mount();
  expect(host.textContent).not.toContain("洞天主动");
  await click("不愧君");
  expect(host.textContent).toContain("洞天主动");
});

it("秘籍与奇穴选择写入草稿并可在重挂载后恢复", async () => {
  vi.mocked(api.loadCombatBundle).mockResolvedValue({
    skills: [skill],
    combat: {
      meta: { profession: "mowen", version: "test", note: "" },
      form: [],
      qifeng: [],
      skill_combat: [{ skill_id: 1, sub_id: 0, miji_ids: ["mj_gong"] }],
      buff: [],
      miji: [{ id: "mj_gong", name: "宫·秘籍", skill_id: 1, sub_id: 0, modifiers: [{ cast_time_add: -0.125 }], effects: ["运功-0.125秒"] }],
      qixue: [{ id: "qx_dongtian", name: "洞天", slot: 1, group_label: "第一重", group_ids: ["qx_dongtian"] }],
    },
  } as never);
  vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], total_damage: 0, window_dps: 0, skipped_actions: 0 });
  await mount();
  await click("测试技·一");
  await click("0/4");
  await click("运功-0.125秒");
  await click("×");
  await click("洞天");
  const saved = JSON.parse(localStorage.getItem(`${STORAGE_KEYS.comboDraft}:mowen`)!).data;
  expect(saved.miji).toEqual(["mj_gong"]);
  expect(saved.qixue).toEqual(["qx_dongtian"]);
  // 重挂载后：秘籍仍在、奇穴仍为选中态
  await mount("mowen", 3);
  const board = host.querySelector('[data-testid="qixue-board"]')!;
  expect(board.querySelector('button[aria-pressed="true"]')?.textContent).toBe("洞天");
  const gear = [...host.querySelectorAll("button")].find((b) => b.textContent === "0/4" || b.textContent === "1/4");
  expect(gear?.textContent).toBe("1/4");
});

it("输入变化清空已完成时间轴，失败可重试且不显示旧伤害", async () => {
  vi.useFakeTimers();
  try {
    vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], steps: [], total_damage: 123456, window_dps: 411, skipped_actions: 0 });
    await mount(); await click("测试技·一");
    await act(async () => { await vi.advanceTimersByTimeAsync(400); });
    expect(host.textContent).toContain("123,456");
    vi.mocked(api.calculateTimeline).mockRejectedValue(new Error("后端拒绝选择"));
    await click("测试技·二");
    expect(host.textContent).not.toContain("123,456");
    await act(async () => { await vi.advanceTimersByTimeAsync(400); });
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("后端拒绝选择");
    vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], steps: [], total_damage: 654321, window_dps: 2181, skipped_actions: 0 });
    await click("重试模拟");
    expect(host.textContent).toContain("654,321");
    expect(host.querySelector('[role="alert"]')).toBeNull();
  } finally { vi.useRealTimers(); }
});

it("输入切换后迟到的成功结果不能覆盖新结果", async () => {
  vi.useFakeTimers();
  try {
    let resolveOld!: (value: Awaited<ReturnType<typeof api.calculateTimeline>>) => void;
    vi.mocked(api.calculateTimeline).mockImplementationOnce(() => new Promise((resolve) => { resolveOld = resolve; }));
    await mount(); await click("测试技·一");
    await act(async () => { await vi.advanceTimersByTimeAsync(400); });
    vi.mocked(api.calculateTimeline).mockResolvedValue({ events: [], steps: [], total_damage: 654321, window_dps: 2181, skipped_actions: 0 });
    await click("测试技·二");
    await act(async () => { await vi.advanceTimersByTimeAsync(400); });
    await act(async () => { resolveOld({ events: [], steps: [], total_damage: 123456, window_dps: 411, skipped_actions: 0 }); });
    expect(host.textContent).toContain("654,321");
    expect(host.textContent).not.toContain("123,456");
  } finally { vi.useRealTimers(); }
});

it("固定槽切换会移除整个奇穴组，技能入口是可聚焦原生按钮", async () => {
 vi.mocked(api.loadCombatBundle).mockResolvedValue({skills: [skill], combat: {meta: {}, form: [], qifeng: [], skill_combat: [], buff: [], miji: [], qixue: [
 {id: "dongquan", name: "冻泉", slot: 4, group_ids: ["dongquan", "dongquan-gong"]},
 {id: "lingdong", name: "凌冬", slot: 4},
 ]}} as never);
 await mount();
 const add = host.querySelector('[data-skill-card]')!;
 expect(add.tagName).toBe("BUTTON");
 (add as HTMLButtonElement).focus(); expect(document.activeElement).toBe(add);
 expect(add.querySelector("button")).toBeNull();
 await click("冻泉"); await click("凌冬");
 const saved = JSON.parse(localStorage.getItem(`${STORAGE_KEYS.comboDraft}:mowen`)!).data;
 expect(saved.qixue).toEqual(["lingdong"]);
});
