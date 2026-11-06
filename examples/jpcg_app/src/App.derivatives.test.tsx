import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import App from "./App";
import { defaultCockpit } from "./state/cockpit";
import * as api from "./api/commands";
import type { SkillResultDTO, ValueSetDTO } from "./types";
import { DEFAULT_COEFFICIENT, STORAGE_KEYS } from "./utils/constants";
vi.mock("./api/commands");
vi.mock("./components/ForumPage", () => ({ default: () => <div>论坛</div> }));
vi.mock("./components/ComboPage", () => ({ default: () => <div>连招</div> }));
vi.mock("./components/SkillEditorPage", () => ({ default: () => <div>编辑器</div> }));
const valueSet: ValueSetDTO = { id: "live", name: "正式服", level: 130, is_default: true, available: true, source: "data", coefficient: DEFAULT_COEFFICIENT };
const result = (q: number): SkillResultDTO[] => [{ skill_name: "测试技", y: 1, b: 2, i: 3, n: q, h: q, q, dot_jumps: [], has_critical_strike: false, zhenshishanghai: 0, lost_hp_zhenshishanghai: 0 }];
let root: Root; let host: HTMLDivElement;
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true }); vi.resetAllMocks(); localStorage.clear();
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
  vi.mocked(api.gearCatalog).mockRejectedValue(new Error("not available"));
  const { loadouts, ...legacy } = defaultCockpit();
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 1, data: { ...legacy, mode: "manual" } }));
  vi.mocked(api.listProfessions).mockResolvedValue([]); vi.mocked(api.loadProfessionConfig).mockResolvedValue(null);
  vi.mocked(api.getModuleVersions).mockResolvedValue({ app: "test", core: "test", combo: "test", update: "test", const: "test" });
  vi.mocked(api.listValueSets).mockResolvedValue([valueSet]); vi.mocked(api.resolveValueSet).mockResolvedValue(valueSet);
  vi.mocked(api.calculateDamage).mockResolvedValue(result(100));
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
async function mount() { await act(async () => root.render(<App />)); }
async function click(text: string) { const button = [...host.querySelectorAll("button")].find((b) => b.textContent === text)!; expect(button).toBeDefined(); await act(async () => button.click()); }
function numberInput(name: string) { return [...host.querySelectorAll("label")].find((l) => l.textContent === name)!.parentElement!.querySelector("input")!; }
async function edit(name: string, value: string) {
  await act(async () => { const input = numberInput(name); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value); input.dispatchEvent(new Event("input", { bubbles: true })); });
}
it("导数页面常驻时，回计算页改属性再返回不会沿用旧推荐", async () => {
  vi.mocked(api.computeDerivatives).mockResolvedValue({ derivatives: [], recommendation: {
    crit_vs_pofang: { better: "旧属性推荐", huixin_total: 1, pofang_total: 2 }, top3: [],
  } });
  await mount(); await edit("基础攻击", "123");
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="加点优化"]')!.click());
  await click("计算导数"); expect(host.textContent).toContain("旧属性推荐");
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="计算"]')!.click());
  await edit("基础攻击", "456");
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="加点优化"]')!.click());
  expect(host.textContent).not.toContain("旧属性推荐");
  expect(api.computeDerivatives).toHaveBeenCalledTimes(1);
  await click("计算导数");
  expect(api.computeDerivatives).toHaveBeenLastCalledWith(expect.objectContaining({ player: expect.objectContaining({ jichu_gongji: 456 }) }));
});
