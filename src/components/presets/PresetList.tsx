import { Button } from "@/components/ui";
import { formatClock } from "@/lib/format";
import type { Preset, PresetMode, Subject } from "@/lib/tauri";

const MODE_LABELS: Record<PresetMode, string> = {
  countup: "Секундомер",
  countdown: "Обратный отсчёт",
  pomodoro: "Помодоро",
};

/** Строка вида «25:00 · 5:00 · 15:00 × 4» — то, что отличает пресеты друг от друга. */
function describe(preset: Preset): string {
  if (preset.mode === "countup") return MODE_LABELS.countup;
  if (preset.mode === "countdown") {
    return `${MODE_LABELS.countdown} · ${formatClock(preset.work_seconds)}`;
  }

  const parts = [
    formatClock(preset.work_seconds),
    formatClock(preset.break_seconds ?? 0),
    formatClock(preset.long_break_seconds ?? 0),
  ].join(" · ");

  return `${MODE_LABELS.pomodoro} · ${parts} × ${preset.cycles_before_long ?? 1}`;
}

type PresetListProps = {
  presets: Preset[];
  subjects: Subject[];
  onEdit: (preset: Preset) => void;
};

/**
 * Список пресетов таймера. Показывает, к какому предмету привязан пресет и
 * какой из них выбран по умолчанию — оба факта определяют, что запустится по
 * кнопке «Старт».
 */
export function PresetList({ presets, subjects, onEdit }: PresetListProps) {
  const subjectNames = new Map(subjects.map((s) => [s.id, s.name]));

  return (
    <ul className="flex flex-col">
      {presets.map((preset) => (
        <li
          key={preset.id}
          className="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-border py-3 last:border-b-0"
        >
          <span className="flex min-w-0 flex-1 flex-col gap-1">
            <span className="flex flex-wrap items-center gap-2 text-15 text-text-1">
              {preset.name}
              {preset.is_default && (
                <span className="rounded-md border border-border-accent px-2 py-0.5 text-11 tracking-label text-accent-text uppercase">
                  По умолчанию
                </span>
              )}
            </span>
            <span className="text-12.5 text-text-dim">
              {describe(preset)}
              {" · "}
              {preset.subject_id
                ? (subjectNames.get(preset.subject_id) ?? "Предмет удалён")
                : "Все предметы"}
            </span>
          </span>

          <Button
            size="sm"
            variant="secondary"
            onClick={() => onEdit(preset)}
            aria-label={`Изменить пресет «${preset.name}»`}
          >
            Изменить
          </Button>
        </li>
      ))}
    </ul>
  );
}
