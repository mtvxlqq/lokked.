import { Screen } from "@/components/Screen";
import { EmptyState } from "@/components/ui";
import { TimerIcon } from "@/components/nav/icons";

/**
 * Раздел «Таймеры» — список предметов и запуск сессии.
 *
 * Заглушка до M5: данные приходят из репозиториев предметов и пресетов,
 * выдумывать их в вёрстке нельзя.
 */
export function Timers() {
  return (
    <Screen title="Таймеры">
      <EmptyState
        icon={<TimerIcon className="size-8" />}
        title="Предметов пока нет"
        description="Здесь появится список предметов с временем за сегодня и кнопкой запуска."
      />
    </Screen>
  );
}
