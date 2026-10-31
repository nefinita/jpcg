import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import App from "./App";
import * as api from "./api/commands";
import type { SkillResultDTO, ValueSetDTO } from "./types";
import { DEFAULT_COEFFICIENT, STORAGE_KEYS } from "./utils/constants";
vi.mock("./api/commands");
vi.mock("./components/ForumPage", () => ({ default: () => <div>论坛</div> }));
vi.mock("./components/ComboPage", () => ({ default: () => <div>连招</div> }));
vi.mock("./components/OptimizePage", () => ({ default: () => <div>导数</div> }));
vi.mock("./components/SkillEditorPage", () => ({ default: () => <div>编辑器</div> }));
const valueSet: ValueSetDTO = { id: "live", name: "正式服", level: 130, is_default: true, available: true, source: "data", coefficient: DEFAULT_COEFFICIENT };
const result = (q: number): SkillResultDTO[] => [{ skill_name: "测试技", y: 1, b: 2, i: 3, n: q, h: q, q, dot_jumps: [], has_critical_strike: false, zhenshishanghai: 0, lost_hp_zhenshishanghai: 0 }];
let root: Root; let host: HTMLDivElement;
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true }); vi.resetAllMocks(); localStorage.clear();
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
  vi.mocked(api.listProfessions).mockResolvedValue([]); vi.mocked(api.loadProfessionConfig).mockResolvedValue(null);
  vi.mocked(api.getModuleVersions).mockResolvedValue({ app: "test", core: "test", combo: "test", update: "test", const: "test" });
  vi.mocked(api.listValueSets).mockResolvedValue([valueSet]); vi.mocked(api.resolveValueSet).mockResolvedValue(valueSet);
  vi.mocked(api.calculateDamage).mockResolvedValue(result(100));
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
async function mount() { await act(async () => root.render(<App />)); }
async function restart() { await act(async () => root.unmount()); root = createRoot(host); await mount(); }
async function click(text: string) { const button = [...host.querySelectorAll("button")].find((b) => b.textContent === text)!; expect(button).toBeDefined(); await act(async () => button.click()); }
function numberInput(name: string) { return [...host.querySelectorAll("label")].find((l) => l.textContent === name)!.parentElement!.querySelector("input")!; }
async function edit(name: string, value: string) {
  await act(async () => { const input = numberInput(name); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value); input.dispatchEvent(new Event("input", { bubbles: true })); });
}
it("输入/敌人/系数/结果/钉住基准重启恢复，恢复基准撤销全部编辑，取消后隐藏差值", async () => {
  await mount(); expect(host.textContent).not.toContain("ΔQ");
  await edit("基础攻击", "123"); await edit("目标血量", "999"); await edit("会心系数", "777");
  await click("开始计算"); await click("钉住基准"); expect(host.textContent).toContain("+0 / +0.0%");
  await edit("基础攻击", "456"); expect(host.textContent).toContain("属性已修改");
  vi.mocked(api.calculateDamage).mockResolvedValue(result(150)); await click("开始计算"); expect(host.textContent).toContain("+50 / +50.0%");
  await restart(); expect(numberInput("基础攻击").value).toBe("456"); expect(numberInput("会心系数").value).toBe("777");
  expect(numberInput("目标血量").value).toBe("999"); expect(host.textContent).toContain("+50 / +50.0%");
  expect(api.calculateDamage).toHaveBeenCalledTimes(2);
  await click("恢复基准"); expect(numberInput("基础攻击").value).toBe("123"); expect(numberInput("会心系数").value).toBe("777");
  expect(host.textContent).toContain("+0 / +0.0%"); await click("取消钉住"); expect(host.textContent).not.toContain("ΔQ");
  await restart(); expect(host.textContent).not.toContain("ΔQ"); expect(numberInput("基础攻击").value).toBe("123");
});
it("未计算的新属性不能钉住旧结果；恢复后忽略迟到的计算响应", async () => {
  await mount(); await click("开始计算"); await edit("基础攻击", "2");
  const pin = [...host.querySelectorAll("button")].find((b) => b.textContent === "钉住基准")!; expect(pin.disabled).toBe(true);
  await click("开始计算"); await click("钉住基准"); await edit("基础攻击", "9");
  let resolve!: (data: SkillResultDTO[]) => void;
  vi.mocked(api.calculateDamage).mockReturnValue(new Promise((done) => { resolve = done; }));
  await click("开始计算"); await click("恢复基准");
  await act(async () => resolve(result(900))); expect(host.textContent).not.toContain("900"); expect(numberInput("基础攻击").value).toBe("2");
});
it("损坏存储安全回退；恢复时不会重新加载心法覆盖已保存修正", async () => {
  localStorage.setItem(STORAGE_KEYS.cockpit, '{"version":1,"data":{"form":{}}}');
  await mount(); await edit("基础攻击", "42");
  const saved = JSON.parse(localStorage.getItem(STORAGE_KEYS.cockpit)!).data;
  saved.form.xinfa_config.atk_up = 23;
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 1, data: saved }));
  vi.mocked(api.loadProfessionConfig).mockResolvedValue({ profession: "mowen", xinfa_name: "莫问", xinfa_nom: "gengu", atk_up: 99, pofang_up: 0, huixin_up: 0 });
  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart(); expect(numberInput("基础攻击").value).toBe("42"); expect(api.loadProfessionConfig).toHaveBeenCalledTimes(calls);
  await click("开始计算"); expect(api.calculateDamage).toHaveBeenLastCalledWith(expect.objectContaining({ xinfa_config: expect.objectContaining({ atk_up: 23 }) }));
});
it("页面访问后常驻，切换页面保留输入，加载配置的心法进入全局状态", async () => {
  await mount(); await edit("基础攻击", "789");
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="技能编辑"]')!.click());
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="计算"]')!.click());
  expect(numberInput("基础攻击").value).toBe("789"); expect(host.textContent).toContain("编辑器");
  expect(host.querySelector('[hidden]')).not.toBeNull();
  const { toCalculateRequest } = await import("./utils/normalize");
  const { defaultManualForm } = await import("./state/cockpit");
  const saved = toCalculateRequest(defaultManualForm());
  saved.xinfa_config = { ...saved.xinfa_config, profession: "qixiu", xinfa_name: "七秀坊", xinfa_nom: "gen" };
  vi.mocked(api.loadConfig).mockResolvedValue(saved);
  await click("加载"); expect(JSON.parse(localStorage.getItem(STORAGE_KEYS.cockpit)!).data.form.xinfa).toBe("qixiu");
  await restart(); expect(host.querySelector("select")!.value).toBe("qixiu");
});

it("首次心法加载失败不把占位表单标记为已初始化，重启会重新加载，成功后不再重复拉取", async () => {
  const storedCockpit = () => JSON.parse(localStorage.getItem(STORAGE_KEYS.cockpit)!).data as { manualReady?: boolean };
  vi.mocked(api.loadProfessionConfig).mockRejectedValue(new Error("boom"));
  await mount();
  expect(storedCockpit().manualReady).toBe(false);

  vi.mocked(api.loadProfessionConfig).mockClear();
  vi.mocked(api.loadProfessionConfig).mockResolvedValue({
    profession: "mowen", xinfa_name: "莫问", xinfa_nom: "gengu", atk_up: 12, pofang_up: 0, huixin_up: 0,
  });
  await restart();
  expect(api.loadProfessionConfig).toHaveBeenCalledTimes(1);
  expect(storedCockpit().manualReady).toBe(true);

  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart();
  expect(api.loadProfessionConfig).toHaveBeenCalledTimes(calls);
});
