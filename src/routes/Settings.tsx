import { Screen } from "@/components/Screen";
import { EmptyState } from "@/components/ui";
import { SettingsIcon } from "@/components/nav/icons";

/**
 * Раздел «Настройки» — граница учебного дня, пресеты таймера, Zen-режим.
 * Заглушка до M5.
 */
export function Settings() {
  return (
    <Screen title="Настройки">
      <EmptyState
        icon={<SettingsIcon className="size-8" />}
        title="Настраивать пока нечего"
        description="Здесь появятся граница учебного дня, пресеты таймера и параметры Zen-режима."
      />
    </Screen>
  );
}
