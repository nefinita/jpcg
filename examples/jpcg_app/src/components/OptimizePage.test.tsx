import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import OptimizePage from "./OptimizePage";
import * as api from "../api/commands";
import { defaultManualForm } from "../state/cockpit";
import type { DerivativesOutputDTO, FormData } from "../types";
vi.mock("../api/commands");
let root: Root;
let host: HTMLDivElement;
const result = (label: string): DerivativesOutputDTO => ({
  derivatives: [],
  recommendation: { crit_vs_pofang: { better: label, huixin_total: 1, pofang_total: 2 }, top3: [] },
});
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  vi.resetAllMocks();
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
async function render(form: FormData | null) { await act(async () => root.render(<OptimizePage formData={form} />)); }
async function calculate() { await act(async () => host.querySelector("button")!.click()); }
const changedForm = (form: FormData): FormData => ({ ...form, player: { ...form.player, jichu_gongji: 12345 } });
it("输入改变后清除已完成的旧导数结果", async () => {
  const form = defaultManualForm();
  vi.mocked(api.computeDerivatives).mockResolvedValue(result("旧推荐"));
  await render(form); await calculate(); expect(host.textContent).toContain("旧推荐");
  await render(changedForm(form)); expect(host.textContent).not.toContain("旧推荐");
});
it("输入改变后丢弃旧请求的迟到成功并允许重新计算", async () => {
  const form = defaultManualForm(); const old = deferred<DerivativesOutputDTO>();
  vi.mocked(api.computeDerivatives).mockReturnValueOnce(old.promise).mockResolvedValueOnce(result("新推荐"));
  await render(form); await calculate(); await render(changedForm(form));
  expect(host.querySelector("button")!.disabled).toBe(false);
  await calculate(); expect(host.textContent).toContain("新推荐");
  await act(async () => old.resolve(result("旧推荐")));
  expect(host.textContent).toContain("新推荐"); expect(host.textContent).not.toContain("旧推荐");
});
it("旧请求失败不会污染新请求的错误与加载状态", async () => {
  const form = defaultManualForm(); const old = deferred<DerivativesOutputDTO>(); const next = deferred<DerivativesOutputDTO>();
  vi.mocked(api.computeDerivatives).mockReturnValueOnce(old.promise).mockReturnValueOnce(next.promise);
  await render(form); await calculate(); await render(changedForm(form)); await calculate();
  await act(async () => old.reject(new Error("旧错误")));
  expect(host.textContent).not.toContain("旧错误"); expect(host.querySelector("button")!.disabled).toBe(true);
  await act(async () => next.resolve(result("新推荐"))); expect(host.textContent).toContain("新推荐");
});
it("配装输入暂不可用再恢复时不复活旧推荐", async () => {
  const form = defaultManualForm();
  vi.mocked(api.computeDerivatives).mockResolvedValue(result("旧推荐"));
  await render(form); await calculate(); await render(null); await render(form);
  expect(host.textContent).not.toContain("旧推荐");
});
it("当前请求的失败仍显示并允许重试", async () => {
  vi.mocked(api.computeDerivatives).mockRejectedValueOnce(new Error("当前错误")).mockResolvedValueOnce(result("重试推荐"));
  await render(defaultManualForm()); await calculate(); expect(host.textContent).toContain("当前错误");
  expect(host.querySelector("button")!.disabled).toBe(false);
  await calculate(); expect(host.textContent).not.toContain("当前错误"); expect(host.textContent).toContain("重试推荐");
});
it("同一输入的普通重渲染保留结果且不重复计算", async () => {
  const form = defaultManualForm();
  vi.mocked(api.computeDerivatives).mockResolvedValue(result("有效推荐"));
  await render(form); await calculate(); await render(form);
  expect(host.textContent).toContain("有效推荐"); expect(api.computeDerivatives).toHaveBeenCalledTimes(1);
});
it("A到B再回A也不能接纳最早A请求", async () => {
  const form = defaultManualForm(); const old = deferred<DerivativesOutputDTO>();
  vi.mocked(api.computeDerivatives).mockReturnValueOnce(old.promise);
  await render(form); await calculate(); await render(changedForm(form)); await render(form);
  await act(async () => old.resolve(result("过期A推荐")));
  expect(host.textContent).not.toContain("过期A推荐"); expect(host.querySelector("button")!.disabled).toBe(false);
});
