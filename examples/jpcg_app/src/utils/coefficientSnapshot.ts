import snapshot from "../../../../crates/jpcg_const/preset/cszj-exp-260908.toml?raw";
import type { CoefficientConfigDTO } from "../types";

const keys = [
  "pofang_xishu", "huixin_xishu", "huixiao_xishu", "yujin_xishu",
  "yuhui_xishu", "huajin_xishu", "fangyu_xishu", "pvp_global_jianshang",
] as const satisfies readonly (keyof CoefficientConfigDTO)[];

// 与 Rust 共用数值真源；仅支持该快照的扁平数值白名单，不是通用 TOML 解析器。
export function parseCoefficientSnapshot(text: string): CoefficientConfigDTO {
  const values: Record<string, number> = {};
  for (const line of text.split(/\r?\n/)) {
    const content = line.split("#", 1)[0].trim();
    if (!content) continue;
    const match = /^(\w+)\s*=\s*(\d+(?:\.\d+)?)$/.exec(content);
    if (!match) throw new Error("系数快照格式错误");
    const [, key, raw] = match;
    const value = Number(raw);
    if (
      (key !== "level" && !keys.some((k) => k === key)) ||
      key in values ||
      !Number.isFinite(value) ||
      (key !== "pvp_global_jianshang" && value <= 0)
    ) {
      throw new Error(`系数快照字段非法：${key}`);
    }
    values[key] = value;
  }
  if (values.level !== 50 || keys.some((key) => !(key in values))) {
    throw new Error("一测系数快照等级错误或缺少字段");
  }
  return Object.fromEntries(keys.map((key) => [key, values[key]])) as unknown as CoefficientConfigDTO;
}

export const FIRST_TEST_COEFFICIENT = parseCoefficientSnapshot(snapshot);
