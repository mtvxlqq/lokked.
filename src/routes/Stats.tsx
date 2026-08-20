import { Screen } from "@/components/Screen";
import { EmptyState } from "@/components/ui";
import { StatsIcon } from "@/components/nav/icons";

/**
 * Раздел «Статистика» — время по предметам, heatmap активности, тренды.
 * Заглушка до M12.
 */
export function Stats() {
  return (
    <Screen title="Статистика">
      <EmptyState
        icon={<StatsIcon className="size-8" />}
        title="Данных пока нет"
        description="Статистика появится после первой завершённой сессии."
      />
    </Screen>
  );
}
