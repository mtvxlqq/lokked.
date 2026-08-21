import { Button } from "@/components/ui";
import { subjectBackground } from "@/components/subjects/palette";
import { formatDuration } from "@/lib/format";
import type { Subject } from "@/lib/tauri";

type SubjectListProps = {
  subjects: Subject[];
  /** Секунды за сегодня по id предмета; предметы без времени здесь отсутствуют. */
  secondsToday: Map<string, number>;
  onStart: (subject: Subject) => void;
  onEdit: (subject: Subject) => void;
};

/**
 * Список предметов: цвет, название, время за сегодня и запуск сессии.
 *
 * Строка переносится, а не сжимается: на 380px название занимает всю ширину,
 * а время и кнопки уходят на вторую строку целиком — так они остаются
 * пригодными для пальца, вместо того чтобы ужиматься до нечитаемого.
 */
export function SubjectList({
  subjects,
  secondsToday,
  onStart,
  onEdit,
}: SubjectListProps) {
  return (
    <ul className="flex flex-col">
      {subjects.map((subject) => (
        <li
          key={subject.id}
          className="flex flex-wrap items-center gap-x-4 gap-y-3 border-b border-border py-3 last:border-b-0"
        >
          <span
            aria-hidden="true"
            className={`h-9 w-1 shrink-0 rounded-full ${subjectBackground(subject.color)}`}
          />

          <span className="min-w-0 flex-1 text-15 text-text-1">
            {subject.icon && (
              <span aria-hidden="true" className="mr-2 text-text-dim">
                {subject.icon}
              </span>
            )}
            {subject.name}
          </span>

          <span className="font-mono text-14 tabular-nums text-text-dim">
            {formatDuration(secondsToday.get(subject.id) ?? 0)}
          </span>

          <span className="flex gap-2.5">
            <Button size="sm" variant="ghost" onClick={() => onStart(subject)}>
              Старт
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => onEdit(subject)}
              aria-label={`Изменить предмет «${subject.name}»`}
            >
              Изменить
            </Button>
          </span>
        </li>
      ))}
    </ul>
  );
}
