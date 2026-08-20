import { Screen } from "@/components/Screen";
import { EmptyState } from "@/components/ui";
import { CardsIcon } from "@/components/nav/icons";

/**
 * Раздел «Карточки» — колоды, редактор и режимы повторения.
 * Заглушка до M9.
 */
export function Cards() {
  return (
    <Screen title="Карточки">
      <EmptyState
        icon={<CardsIcon className="size-8" />}
        title="Колод пока нет"
        description="Здесь появятся колоды, редактор карточек и режимы повторения."
      />
    </Screen>
  );
}
