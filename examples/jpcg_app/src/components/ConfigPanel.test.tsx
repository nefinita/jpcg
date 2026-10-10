import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import ConfigPanel from "./ConfigPanel";
import * as api from "../api/commands";
import { DEFAULT_BUFF, DEFAULT_COEFFICIENT, STORAGE_KEYS } from "../utils/constants";
import type { CalculateRequest, FormData, ValueSetDTO } from "../types";

vi.mock("../api/commands");

const live: ValueSetDTO = {
  id: "live-130", name: "正式服130", level: 130, is_default: true,
  source: "data", available: true,
  coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 130 },
};
const experience: ValueSetDTO = {
  ...live, id: "experience-140", name: "体验服140", level: 140, is_default: false,
  coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 140 },
};
const saved = (value_set: string | null = live.id): CalculateRequest => ({
  player: { jcsx: "gengu", jichu_shuxing: 1, jichu_gongji: 2, huixin_dengji: 3, huixin_xiaoguo: 4, pofang_dengji: 5, wuqi_shanghai: 6 },
  hostile: { waigong_fangyu: 1, neigong_fangyu: 2, yujin_dengji: 3, huajin_dengji: 4, jianshang_bili: 5, target_hp: 6, max_hp: 6, current_hp: 6 },
  xinfa_config: { profession: "mowen", xinfa_name: "莫问", xinfa_nom: "gengu", atk_up: 0, pofang_up: 0, huixin_up: 0 },
  buff: { ...DEFAULT_BUFF },
  coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 999 },
  value_set,
});
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

let container: HTMLDivElement;
let root: Root;
const onCalculate = vi.fn<(form: FormData) => void>();
const addToast = vi.fn();
const setStatus = vi.fn();

async function mount() {
  await act(async () => {
    root.render(<ConfigPanel calculating={false} onCalculate={onCalculate} addToast={addToast} setStatus={setStatus} />);
  });
}
async function click(label: string) {
  const button = Array.from(container.querySelectorAll("button")).find((el) => el.textContent === label);
  expect(button, `button ${label}`).toBeDefined();
  await act(async () => { button!.click(); });
}
function selection() { return container.querySelectorAll("select")[1]; }
async function select(id: string) {
  await act(async () => {
    selection().value = id;
    selection().dispatchEvent(new Event("change", { bubbles: true }));
  });
}
function coefficientInput(name = "会心系数") {
  const label = Array.from(container.querySelectorAll("label")).find((el) => el.textContent === name)!;
  return label.parentElement!.querySelector("input")!;
}
async function editCoefficient(value: string) {
  const input = coefficientInput();
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}
async function request() {
  const count = onCalculate.mock.calls.length;
  await click("开始计算");
  expect(onCalculate).toHaveBeenCalledTimes(count + 1);
  return onCalculate.mock.calls[onCalculate.mock.calls.length - 1][0];
}
function applied(name: string) {
  expect(container.textContent).toContain(`实际使用：${name}`);
}

beforeEach(() => {
  vi.resetAllMocks();
  localStorage.clear();
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
  vi.mocked(api.listProfessions).mockResolvedValue([]);
  vi.mocked(api.getModuleVersions).mockResolvedValue({ app: "test", core: "test", combo: "test", update: "test", const: "test" });
  vi.mocked(api.loadProfessionConfig).mockResolvedValue(null);
  vi.mocked(api.listValueSets).mockResolvedValue([live, experience]);
  vi.mocked(api.resolveValueSet).mockImplementation(async (id) => id === experience.id ? experience : live);
  vi.mocked(api.loadConfig).mockResolvedValue(saved());
  vi.mocked(api.checkUpdate).mockResolvedValue({
    current_app_version: null, latest_app_version: null, has_app_update: false,
    current_data_version: null, latest_data_version: null, has_data_update: true,
    data_files_to_update: ["values/index.toml"], has_modules_update: false,
    modules_version: null, modules_files_to_update: [],
  });
  vi.mocked(api.performUpdate).mockResolvedValue("ok");
  vi.mocked(api.listenUpdateProgress).mockReturnValue(vi.fn());
});
afterEach(async () => {
  await act(async () => root.unmount());
  container.remove();
});

describe("ConfigPanel 当前数值集", () => {
  it("选择体验服后加载正式服，清空仍解析正式服并重新填充系数", async () => {
    await mount();
    await select(experience.id);
    await click("加载");
    expect(selection().value).toBe(live.id);
    expect(localStorage.getItem(STORAGE_KEYS.valueSet)).toBe(live.id);
    applied(live.name);
    expect((await request()).coefficient.huixin_xishu).toBe(999);
    // 外部偏好变化也不能替代当前表单选择。
    localStorage.setItem(STORAGE_KEYS.valueSet, experience.id);
    await click("清空");
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(live.id);
    expect(localStorage.getItem(STORAGE_KEYS.valueSet)).toBe(live.id);
    applied(live.name);
    const form = await request();
    expect(form.value_set).toBe(live.id);
    expect(form.coefficient).toEqual(live.coefficient);
    expect(form.player.jichu_shuxing).toBe(0);
    await click("保存");
    expect(api.saveConfig).toHaveBeenLastCalledWith(expect.objectContaining({ value_set: live.id, coefficient: live.coefficient }));
  });

  it("加载后数据更新使用当前正式服，保留存档自定义系数", async () => {
    await mount();
    await select(experience.id);
    await click("加载");
    localStorage.setItem(STORAGE_KEYS.valueSet, experience.id);
    await click("检查更新");
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(live.id);
    applied(live.name);
    expect(await request()).toMatchObject({ value_set: live.id, coefficient: saved().coefficient });
  });

  it.each([null, undefined])("旧存档的 %s 选择使用默认解析，不继承上一次体验服", async (id) => {
    await mount();
    await select(experience.id);
    vi.mocked(api.loadConfig).mockResolvedValue({ ...saved(), value_set: id });
    await click("加载");
    expect(selection().value).toBe("");
    expect(localStorage.getItem(STORAGE_KEYS.valueSet)).toBeNull();
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(null);
    expect((await request()).value_set).toBeNull();
    await click("检查更新");
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(null);
    expect((await request()).coefficient).toEqual(saved().coefficient);
    await click("清空");
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(null);
    expect(await request()).toMatchObject({ value_set: null, coefficient: live.coefficient });
    applied(live.name);
  });

  it("快速切换时迟到的体验服解析不能覆盖正式服", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockImplementation(async (id) => id === experience.id ? pending.promise : live);
    await select(experience.id);
    expect(selection().value).toBe(experience.id);
    expect(container.textContent).not.toContain("实际使用：");
    await select(live.id);
    await act(async () => pending.resolve(experience));
    applied(live.name);
    expect(await request()).toMatchObject({ value_set: live.id, coefficient: live.coefficient });
  });

  it("加载使先前未完成的选择解析失效", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockImplementation(async (id) => id === experience.id ? pending.promise : live);
    await select(experience.id);
    await click("加载");
    await act(async () => pending.resolve(experience));
    applied(live.name);
    expect(await request()).toMatchObject({ value_set: live.id, coefficient: saved().coefficient });
  });

  it("加载的解析或存档请求迟到时不能覆盖后续选择", async () => {
    await mount();
    const pendingResolve = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockImplementation(async (id) => id === live.id ? pendingResolve.promise : experience);
    await click("加载");
    await select(experience.id);
    await act(async () => pendingResolve.resolve(live));
    applied(experience.name);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
    const pendingLoad = deferred<CalculateRequest | null>();
    vi.mocked(api.loadConfig).mockReturnValue(pendingLoad.promise);
    await click("加载");
    await select(experience.id);
    await act(async () => pendingLoad.resolve(saved()));
    applied(experience.name);
    expect(localStorage.getItem(STORAGE_KEYS.valueSet)).toBe(experience.id);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
  });

  it("数据刷新解析迟到时不能覆盖随后切换的体验服", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockImplementation(async (id) => id === live.id ? pending.promise : experience);
    await click("检查更新");
    await select(experience.id);
    await act(async () => pending.resolve(live));
    applied(experience.name);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
  });

  it("热更新重新填充默认系数，但保留解析等待期间的手工编辑", async () => {
    await mount();
    const updated = { ...live, coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 131 } };
    vi.mocked(api.resolveValueSet).mockResolvedValue(updated);
    await click("检查更新");
    expect((await request()).coefficient).toEqual(updated.coefficient);
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValue(pending.promise);
    await click("检查更新");
    expect(coefficientInput().disabled).toBe(false);
    await editCoefficient("777");
    await act(async () => pending.resolve(updated));
    expect((await request()).coefficient.huixin_xishu).toBe(777);
  });

  it("不可用值集回退只更改实际标签和系数，保留请求 id", async () => {
    await mount();
    vi.mocked(api.resolveValueSet).mockResolvedValue(live);
    await select(experience.id);
    expect(selection().value).toBe(experience.id);
    applied(live.name);
    expect(container.textContent).toContain("（已回退）");
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: live.coefficient });
  });

  it("启动列表迟到不能覆盖已经加载的存档", async () => {
    const pending = deferred<ValueSetDTO[]>();
    vi.mocked(api.listValueSets).mockReturnValue(pending.promise);
    await mount();
    vi.mocked(api.loadConfig).mockResolvedValue(saved(experience.id));
    await click("加载");
    await act(async () => pending.resolve([live, experience]));
    applied(experience.name);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: saved().coefficient });
  });

  it("同一选择的热更新解析结果优先于更早的解析", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    const updated = { ...experience, coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 141 } };
    vi.mocked(api.resolveValueSet).mockReturnValueOnce(pending.promise).mockResolvedValue(updated);
    await select(experience.id);
    await click("检查更新");
    expect((await request()).coefficient).toEqual(updated.coefficient);
    await act(async () => pending.resolve(experience));
    applied(experience.name);
    expect((await request()).coefficient).toEqual(updated.coefficient);
  });

  it.each([false, true])("加载无存档/失败 (%s) 后继续解析当前选择", async (fails) => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValueOnce(pending.promise).mockResolvedValue(experience);
    await select(experience.id);
    if (fails) vi.mocked(api.loadConfig).mockRejectedValue(new Error("read failed"));
    else vi.mocked(api.loadConfig).mockResolvedValue(null);
    await click("加载");
    applied(experience.name);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
    await act(async () => pending.resolve(live));
    applied(experience.name);
    expect((await request()).coefficient).toEqual(experience.coefficient);
  });


  it("选择解析前禁用计算和保存，避免新 id 配旧系数", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValue(pending.promise);
    await select(experience.id);
    const button = (name: string) => Array.from(container.querySelectorAll("button")).find((el) => el.textContent === name)!;
    expect(button("开始计算").disabled).toBe(true);
    expect(button("保存").disabled).toBe(true);
    await click("开始计算");
    await click("保存");
    expect(onCalculate).not.toHaveBeenCalled();
    expect(api.saveConfig).not.toHaveBeenCalled();
    await act(async () => pending.resolve(experience));
    expect(button("开始计算").disabled).toBe(false);
    expect(button("保存").disabled).toBe(false);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
  });


  it("显式选择默认值集不误报 null 不可用或回退", async () => {
    await mount();
    await select(experience.id);
    addToast.mockClear();
    await select("");
    expect(api.resolveValueSet).toHaveBeenLastCalledWith(null);
    expect(localStorage.getItem(STORAGE_KEYS.valueSet)).toBeNull();
    expect(addToast).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("（已回退）");
    expect(await request()).toMatchObject({ value_set: null, coefficient: live.coefficient });
  });


  it.each(["切换", "清空"])("%s等待新系数基线时不能编辑单字段留下其他旧系数", async (action) => {
    await mount();
    const next = { ...experience, coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 140, pofang_xishu: 10378.17 } };
    if (action === "清空") {
      vi.mocked(api.resolveValueSet).mockResolvedValue(next);
      await select(experience.id);
      await editCoefficient("888");
    }
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValue(pending.promise);
    if (action === "切换") await select(experience.id);
    else await click("清空");
    expect(coefficientInput().disabled).toBe(true);
    expect(coefficientInput("破防系数").disabled).toBe(true);
    // 即使合成/排队的 input 事件到达，也不能跳过整套新基线。
    await editCoefficient("777");
    await act(async () => pending.resolve(next));
    expect(coefficientInput().disabled).toBe(false);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: next.coefficient });
    await click("保存");
    expect(api.saveConfig).toHaveBeenLastCalledWith(expect.objectContaining({ value_set: experience.id, coefficient: next.coefficient }));
    await editCoefficient("666");
    expect((await request()).coefficient.huixin_xishu).toBe(666);
  });


  it("存档基线已知时不因等待实际值集标签而锁定系数", async () => {
    await mount();
    const pending = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValue(pending.promise);
    await select(experience.id);
    expect(coefficientInput().disabled).toBe(true);
    await click("加载");
    expect(coefficientInput().disabled).toBe(false);
    await editCoefficient("777");
    await act(async () => pending.resolve(live));
    expect(await request()).toMatchObject({
      value_set: live.id,
      coefficient: { ...saved().coefficient, huixin_xishu: 777 },
    });
  });

  it("旧解析不得解锁新基线，失败后保持锁定直到重试成功", async () => {
    await mount();
    const old = deferred<ValueSetDTO>();
    const latest = deferred<ValueSetDTO>();
    vi.mocked(api.resolveValueSet).mockReturnValueOnce(old.promise).mockReturnValueOnce(latest.promise);
    await select(experience.id);
    await select(live.id);
    await act(async () => old.resolve(experience));
    expect(coefficientInput().disabled).toBe(true);
    await act(async () => latest.resolve(live));
    expect(coefficientInput().disabled).toBe(false);
    vi.mocked(api.resolveValueSet).mockRejectedValueOnce(new Error("resolve failed"));
    await select(experience.id);
    expect(coefficientInput().disabled).toBe(true);
    vi.mocked(api.resolveValueSet).mockResolvedValue(experience);
    await click("清空");
    expect(coefficientInput().disabled).toBe(false);
    expect(await request()).toMatchObject({ value_set: experience.id, coefficient: experience.coefficient });
  });

});
