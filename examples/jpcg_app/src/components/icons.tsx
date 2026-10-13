import type { SVGProps } from "react";

// 自绘 SVG 线条图标库（薄荷/松石绿主题，stroke=currentColor 继承 CSS 着色）
// 统一 viewBox 24，line 图标，圆角线帽。不依赖第三方图标库。

interface IconProps extends SVGProps<SVGSVGElement> {
  size?: number;
}

function base(props: IconProps, size?: number) {
  const { size: s = size ?? 18, ...rest } = props;
  return {
    width: s,
    height: s,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
    ...rest,
  };
}

/** 计算 / 数据（表格+折线） */
export function IconCalc(props: IconProps) {
  return (
    <svg {...base(props)}>
      <rect x="4" y="3.5" width="16" height="17" rx="2.5" />
      <path d="M8 8.5h8M8 12h8M8 15.5h4" />
    </svg>
  );
}

/** 论坛 / 地球 */
export function IconGlobe(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="8.5" />
      <path d="M3.5 12h17M12 3.5c2.5 2.3 3.5 5.2 3.5 8.5s-1 6.2-3.5 8.5c-2.5-2.3-3.5-5.2-3.5-8.5s1-6.2 3.5-8.5z" />
    </svg>
  );
}

/** 排轴器 / 链条连接 */
export function IconCombo(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M10 13a4 4 0 0 0 5.6.4l2.8-2.8a4 4 0 0 0-5.6-5.6l-1.5 1.5" />
      <path d="M14 11a4 4 0 0 0-5.6-.4l-2.8 2.8a4 4 0 0 0 5.6 5.6l1.5-1.5" />
    </svg>
  );
}

/** 加点优化 / 趋势上升 */
export function IconTrend(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M3.5 17.5 10 11l4 4 6.5-7.5" />
      <path d="M14.5 7.5h6v6" />
    </svg>
  );
}

/** 技能编辑 / 铅笔 */
export function IconPencil(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M4 20l1-4.5L16.5 4a2.1 2.1 0 0 1 3 3L8 18.5 4 20z" />
      <path d="M14.5 6l3 3" />
    </svg>
  );
}

/** 展开/折叠 左箭头 */
export function IconChevronLeft(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M14.5 6 9 12l5.5 6" />
    </svg>
  );
}

/** 展开/折叠 右箭头 */
export function IconChevronRight(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M9.5 6 15 12l-5.5 6" />
    </svg>
  );
}

/** 主题：太阳 */
export function IconSun(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5 5l1.4 1.4M17.6 17.6 19 19M19 5l-1.4 1.4M6.4 17.6 5 19" />
    </svg>
  );
}

/** 主题：月亮 */
export function IconMoon(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5z" />
    </svg>
  );
}

/** 设置 / 齿轮 */
export function IconGear(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5 5l1.6 1.6M17.4 17.4 19 19M19 5l-1.6 1.6M6.6 17.4 5 19" />
    </svg>
  );
}

/** 删除 / 垃圾桶 */
export function IconTrash(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M4 6.5h16M9 6.5V5a1.5 1.5 0 0 1 1.5-1.5h3A1.5 1.5 0 0 1 15 5v1.5M6 6.5l.8 12A2 2 0 0 0 8.8 20h6.4a2 2 0 0 0 2-1.5l.8-12" />
      <path d="M10 10.5v6M14 10.5v6" />
    </svg>
  );
}

/** 保存 / 软盘 */
export function IconSave(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M5 3.5h11l3.5 3.5v13.5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4.5a1 1 0 0 1 1-1z" />
      <path d="M8 3.5V8h7V3.5M8.5 20v-5h7v5" />
    </svg>
  );
}

/** 收藏 / 星标 */
export function IconStar(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M12 3.5 14.6 9l6 .6-4.5 4 .1 6-4.2-2.5L7.8 19.6l.1-6-4.5-4 6-.6z" />
    </svg>
  );
}

/** 关闭 / X */
export function IconClose(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M6 6l12 12M18 6 6 18" />
    </svg>
  );
}

/** Toast 成功 / 对勾 */
export function IconCheck(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M5 12.5l4.5 4.5L19 7" />
    </svg>
  );
}

/** Toast 警告 / 感叹号 */
export function IconAlert(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M12 3.5 21 20H3z" />
      <path d="M12 10v4M12 17.5v.1" />
    </svg>
  );
}

/** 反馈 / 虫子 */
export function IconBug(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M9 4 7.5 2.5M15 4l1.5-1.5" />
      <rect x="8" y="6" width="8" height="13" rx="3" />
      <path d="M8 10H4.5M15.5 10H19.5M8 14.5H4.5M15.5 14.5H19.5M9 7 7.5 5.5M15 7l1.5-1.5" />
    </svg>
  );
}
