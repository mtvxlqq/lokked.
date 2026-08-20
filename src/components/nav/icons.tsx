import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/**
 * Иконки навигации. Один стиль на все: сетка 24, только обводка 1.6, скруглённые
 * концы, никакой заливки. Цвет берётся у родителя через `stroke-current`,
 * поэтому активное состояние задаётся обычной утилитой цвета текста.
 */
function Icon({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={cn("size-5.5 stroke-current", className)}
    >
      {children}
    </svg>
  );
}

export type IconProps = { className?: string };

export function TimerIcon({ className }: IconProps) {
  return (
    <Icon className={className}>
      <circle cx="12" cy="13" r="8" />
      <path d="M12 9.5V13l2.4 1.6" />
      <path d="M9.5 2.6h5" />
    </Icon>
  );
}

export function CardsIcon({ className }: IconProps) {
  return (
    <Icon className={className}>
      <rect x="3.2" y="7.6" width="14" height="12" rx="2.4" />
      <path d="M7 4.6h11a2.4 2.4 0 0 1 2.4 2.4v9" />
    </Icon>
  );
}

export function StatsIcon({ className }: IconProps) {
  return (
    <Icon className={className}>
      <path d="M4.4 19.6h15.2" />
      <path d="M7.6 19.6v-6.4" />
      <path d="M12 19.6V6.4" />
      <path d="M16.4 19.6v-9.4" />
    </Icon>
  );
}

export function SettingsIcon({ className }: IconProps) {
  return (
    <Icon className={className}>
      <path d="M4 7.6h8.4" />
      <path d="M17.6 7.6H20" />
      <circle cx="15" cy="7.6" r="2.4" />
      <path d="M4 16.4h2.4" />
      <path d="M11.6 16.4H20" />
      <circle cx="9" cy="16.4" r="2.4" />
    </Icon>
  );
}
