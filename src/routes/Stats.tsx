import { useState } from "react";

import { Screen } from "@/components/Screen";
import { CardTab, type CardSelection } from "@/components/stats/CardTab";
import { CardsTab } from "@/components/stats/CardsTab";
import { ExportDialog } from "@/components/stats/ExportDialog";
import { TimeTab } from "@/components/stats/TimeTab";
import { Button, SegmentedControl } from "@/components/ui";
import type { StatsRange } from "@/lib/tauri";

type Tab = "time" | "cards" | "card";

const TABS: { value: Tab; label: string }[] = [
  { value: "time", label: "Время" },
  { value: "cards", label: "Карточки" },
  { value: "card", label: "Карточка" },
];

const RANGES: { value: StatsRange; label: string }[] = [
  { value: "day", label: "День" },
  { value: "week", label: "Неделя" },
  { value: "month", label: "Месяц" },
  { value: "all", label: "Всё время" },
];

/**
 * Раздел «Статистика»: время по предметам, точность карточек и разбор одной
 * карточки.
 *
 * Экран сам ничего не считает — каждая вкладка спрашивает свою команду.
 * Период общий для первых двух: переключение вкладки не сбрасывает то, что
 * студент уже выбрал.
 *
 * У вкладки «Карточка» периода нет: история карточки — это вся её история,
 * а не выборка за неделю.
 */
export function Stats() {
  const [tab, setTab] = useState<Tab>("time");
  const [range, setRange] = useState<StatsRange>("week");
  const [selection, setSelection] = useState<CardSelection | null>(null);
  const [exporting, setExporting] = useState(false);

  return (
    <Screen
      title="Статистика"
      actions={
        tab !== "card" && (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setExporting(true)}
          >
            Экспорт в CSV
          </Button>
        )
      }
    >
      <div className="flex flex-col gap-3">
        <SegmentedControl
          label="Раздел статистики"
          value={tab}
          options={TABS}
          onChange={setTab}
        />

        {tab !== "card" && (
          <SegmentedControl
            label="Период"
            value={range}
            options={RANGES}
            onChange={setRange}
          />
        )}
      </div>

      {tab === "time" && <TimeTab range={range} />}

      {tab === "cards" && (
        <CardsTab
          range={range}
          onOpenCard={(card) => {
            setSelection({ deckId: card.deck_id, cardId: card.card_id });
            setTab("card");
          }}
        />
      )}

      {tab === "card" && (
        <CardTab selection={selection} onSelect={setSelection} />
      )}

      {exporting && (
        <ExportDialog open range={range} onClose={() => setExporting(false)} />
      )}
    </Screen>
  );
}
