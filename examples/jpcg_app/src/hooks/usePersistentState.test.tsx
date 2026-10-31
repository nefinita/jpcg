import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { readPersistentState, usePersistentState } from "./usePersistentState";

const schema = { version: 1, validate: (x: unknown): x is number => typeof x === "number", migrate: (x: unknown, v: number) => v === 0 ? x : null };
const fallback = () => 0;
let root: Root;
let host: HTMLDivElement;
let oldSet: ((n: number) => void) | undefined;
function Harness({ name }: { name: string }) {
  const [n, setN] = usePersistentState(name, fallback, schema);
  if (name === "a") oldSet = setN;
  return <button onClick={() => setN((v) => v + 1)}>{n}</button>;
}
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  localStorage.clear(); host = document.createElement("div"); root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); vi.restoreAllMocks(); });
const mount = async (name = "a") => { await act(async () => root.render(<Harness name={name} />)); };
const click = async () => { await act(async () => host.querySelector("button")!.click()); };
it("迁移无版本旧值，损坏/错误 schema/未来版本安全回退", () => {
  localStorage.setItem("old", "4"); expect(readPersistentState("old", fallback, schema)).toBe(4);
  for (const bad of ['{', '{"version":1,"data":{}}', '{"version":2,"data":9}']) {
    localStorage.setItem("bad", bad); expect(readPersistentState("bad", fallback, schema)).toBe(0);
  }
});
it("更新可重启恢复，快速换 key 不串写，旧异步 setter 不修改新 key", async () => {
  await mount(); await click(); expect(JSON.parse(localStorage.getItem("a")!).data).toBe(1);
  const previous = oldSet!;
  await mount("b"); expect(host.textContent).toBe("0"); await click(); await click();
  await act(async () => previous(99)); expect(host.textContent).toBe("2");
  await mount("a"); expect(host.textContent).toBe("1");
  await act(async () => root.unmount()); root = createRoot(host); await mount("b"); expect(host.textContent).toBe("2");
});
it("未知未来版本在未编辑前不被默认值覆盖", async () => {
  const future = '{"version":2,"data":9}'; localStorage.setItem("a", future);
  await mount(); expect(host.textContent).toBe("0"); expect(localStorage.getItem("a")).toBe(future);
});
it("存储读取/写入被禁用时仍可在内存编辑", async () => {
  vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("quota"); });
  await mount(); await click(); expect(host.textContent).toBe("1");
});
