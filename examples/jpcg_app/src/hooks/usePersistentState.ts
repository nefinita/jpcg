import { useCallback, useEffect, useState, type SetStateAction } from "react";

export function readPreference(key: string): string | null {
  try { return localStorage.getItem(key); } catch { return null; }
}
export function writePreference(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch { /* 存储被禁用或已满时仍可继续使用内存状态。 */ }
}

interface Schema<T> {
  version: number;
  validate: (value: unknown) => value is T;
  migrate?: (value: unknown, version: number) => unknown;
}

export function readPersistentState<T>(key: string, fallback: () => T, schema: Schema<T>): T {
  try {
    const raw: unknown = JSON.parse(readPreference(key) ?? "null");
    if (raw === null) return fallback();
    const envelope = typeof raw === "object" && "version" in raw && "data" in raw
      ? raw as { version: number; data: unknown } : { version: 0, data: raw };
    // 未知的未来版本不可降级解释。
    if (envelope.version > schema.version) return fallback();
    const value = envelope.version === schema.version ? envelope.data
      : schema.migrate?.(envelope.data, envelope.version);
    return schema.validate(value) ? value : fallback();
  } catch { return fallback(); }
}

/** key 可变；旧心法的迟到回调不会写入新心法草稿。schema/fallback 应为稳定引用。 */
export function usePersistentState<T>(key: string, fallback: () => T, schema: Schema<T>) {
  const [entry, setEntry] = useState(() => ({ key, value: readPersistentState(key, fallback, schema), dirty: false }));
  let current = entry;
  if (entry.key !== key) {
    current = { key, value: readPersistentState(key, fallback, schema), dirty: false };
    setEntry(current);
  }
  useEffect(() => {
    if (!current.dirty) return;
    writePreference(current.key, JSON.stringify({ version: schema.version, data: current.value }));
  }, [current, schema.version]);
  const setValue = useCallback((update: SetStateAction<T>) => {
    setEntry((previous) => previous.key !== key ? previous : ({
      key, dirty: true,
      value: typeof update === "function" ? (update as (value: T) => T)(previous.value) : update,
    }));
  }, [key]);
  return [current.value, setValue] as const;
}
