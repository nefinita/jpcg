export function sanitizeNumbers(obj: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === "string" && value.trim() === "") {
      result[key] = 0;
    } else if (typeof value === "string" && isNaN(Number(value))) {
      result[key] = value;
    } else {
      const num = Number(value);
      result[key] = isNaN(num) ? 0 : num;
    }
  }
  return result;
}
