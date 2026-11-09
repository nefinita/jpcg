// 由 #94 synthetic-v1 目录通过真实 gear cdylib 生成；仅测试使用，不进入浏览器生产包。
import data from "./gear-contract.json";
import type { GearCatalog, GearComposition } from "../types/gear";
export const fixture = data as unknown as { catalog: GearCatalog; cases: GearComposition[] };
