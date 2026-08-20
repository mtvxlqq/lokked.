import type { ComponentType } from "react";

import {
  CardsIcon,
  SettingsIcon,
  StatsIcon,
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
 * Разделы приложения в порядке макета. «Серия» и «Группы» из макета появятся
 * вместе со своими этапами (M18 и M21) — пустых пунктов навигации в меню быть
 * не должно.
 */
export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Таймеры", Icon: TimerIcon },
  { to: "/cards", label: "Карточки", Icon: CardsIcon },
  { to: "/stats", label: "Статистика", Icon: StatsIcon },
  { to: "/settings", label: "Настройки", Icon: SettingsIcon },
];
