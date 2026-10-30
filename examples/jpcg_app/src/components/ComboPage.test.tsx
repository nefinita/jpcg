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
  vi.mocked(api.loadSkillPool).mockResolvedValue([skill, { ...skill, skill_name: "测试技·二", sub_id: 1 }]);
  vi.mocked(api.listComboPresets).mockResolvedValue(["已保存"]);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
async function mount(xinfa = "mowen", revision = 0) { await act(async () => root.render(<ComboPage key={`${xinfa}-${revision}`} xinfaName={xinfa} formData={form} />)); }
async function click(text: string) {
  const button = [...host.querySelectorAll("button")].find((b) => b.textContent === text || b.title === text)!;
  expect(button).toBeDefined(); await act(async () => button.click());
}
async function input(el: HTMLInputElement, value: string) {
  await act(async () => { Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(el, value); el.dispatchEvent(new Event("input", { bubbles: true })); });
}
it("草稿序列/步调整/名称/折叠/搜索按心法隔离，重挂载恢复且不自动覆盖预设", async () => {
  await mount(); await click("测试技·一"); await click("调整系数");
  await input(host.querySelector('input[type="number"]')!, "123"); await click("保存调整");
  await click(" 保存"); await input(host.querySelector('input[placeholder="连招名称"]')!, "未完成草稿");
  await click("取消"); await click("折叠");
  await input(host.querySelector('input[type="search"]')!, "测试");
  const saved = JSON.parse(localStorage.getItem(`${STORAGE_KEYS.comboDraft}:mowen`)!).data;
  expect(saved.sequence[0].overrides.base_damage_override).toBe(123);
  expect(saved.presetName).toBe("未完成草稿"); expect(saved.collapsedGroups).toEqual(["测试技"]);
  await mount("qixiu"); expect(host.textContent).toContain("从下方技能池添加技能"); await click("测试技·二");
  await mount("mowen", 1); expect(host.querySelector('input[type="search"]')!.getAttribute("value")).toBe("测试");
  expect(host.textContent).toContain("测试技·一"); expect(host.textContent).not.toContain("测试技·二");
  await click(" 保存"); expect((host.querySelector('input[placeholder="连招名称"]') as HTMLInputElement).value).toBe("未完成草稿");
  expect(api.saveComboPreset).not.toHaveBeenCalled(); expect(api.deleteComboPreset).not.toHaveBeenCalled();
});
it("加载预设只更新当前草稿，修改后不自动写回预设", async () => {
  vi.mocked(api.loadComboPreset).mockResolvedValue({ name: "已保存", steps: [{ skill, overrides: null }] });
  await mount();
  await act(async () => { const select = host.querySelector("select")!; select.value = "已保存"; select.dispatchEvent(new Event("change", { bubbles: true })); });
  await click("清空"); await mount("mowen", 1);
  expect(host.textContent).toContain("从下方技能池添加技能"); expect(host.querySelector("select")!.value).toBe("已保存");
  expect(api.saveComboPreset).not.toHaveBeenCalled();
});
