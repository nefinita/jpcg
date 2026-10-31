import { useEffect, useRef, useState } from "react";
import { formatNumber } from "../utils/format";

export default function AnimatedNumber({ value }: { value: number }) {
  const [shown, setShown] = useState(value);
  const current = useRef(value);
  useEffect(() => {
    const from = current.current;
    if (!window.matchMedia || window.matchMedia("(prefers-reduced-motion: reduce)").matches || !window.requestAnimationFrame) {
      current.current = value; setShown(value); return;
    }
    let frame = 0;
    const start = performance.now();
    const tick = (now: number) => {
      const progress = Math.min(1, (now - start) / 180);
      current.current = from + (value - from) * (1 - (1 - progress) ** 3);
      setShown(Math.round(current.current));
      if (progress < 1) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [value]);
  return <span aria-label={formatNumber(value)}>{formatNumber(shown)}</span>;
}
