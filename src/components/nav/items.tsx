import type { ComponentType } from "react";

import {
  CardsIcon,
  SettingsIcon,
  StatsIcon,
  StreakIcon,
  TimerIcon,
  type IconProps,
} from "@/components/nav/icons";

export type NavItem = {
  /** Путь маршрута; пустая строка — индексный. */
  to: string;
  label: string;
  Icon: ComponentType<IconProps>;
};

/**
 * Разделы приложения в порядке макета. «Группы» появятся вместе со своим
 * этапом (M21) — пустых пунктов навигации в меню быть не должно.
 */
export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Таймеры", Icon: TimerIcon },
  { to: "/cards", label: "Карточки", Icon: CardsIcon },
  { to: "/streak", label: "Серия", Icon: StreakIcon },
  { to: "/stats", label: "Статистика", Icon: StatsIcon },
  { to: "/settings", label: "Настройки", Icon: SettingsIcon },
];
