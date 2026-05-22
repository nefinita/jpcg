export function formatNumber(n: number): string {
  if (n >= 100_000_000) {
    return (n / 100_000_000).toFixed(2) + "亿";
  }
  if (n >= 10_000) {
    return (n / 10_000).toFixed(1) + "万";
  }
  return n.toLocaleString("zh-CN");
}

export function formatPercent(n: number): string {
  return (n * 100).toFixed(1) + "%";
}
