import { subjectBackground } from "@/components/subjects/palette";
import { formatDuration } from "@/lib/format";
import type { Subject, SubjectTotal } from "@/lib/tauri";

type SubjectBarsProps = {
  totals: SubjectTotal[];
  subjects: Subject[];
};

/**
 * Время по предметам: строка на предмет, полоса длиной от самого большого.
 *
 * Длина сравнивается с лидером, а не с суммой: так видно, во сколько раз
 * один предмет обогнал другой, а это и есть вопрос, который задают этому
 * графику.
 *
 * Предмет, которого больше нет в списке (удалён после того, как время было
 * записано), всё равно показывается: удалять его время задним числом было бы
 * враньём.
 */
export function SubjectBars({ totals, subjects }: SubjectBarsProps) {
  const names = new Map(subjects.map((subject) => [subject.id, subject]));

  return (
    <ul className="flex flex-col gap-3.5">
      {totals.map((total) => {
        const subject = names.get(total.subject_id);

        return (
          <li key={total.subject_id} className="flex flex-col gap-1.5">
            <div className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 truncate text-14 text-text-2">
                {subject?.name ?? "Удалённый предмет"}
              </span>
              <span className="shrink-0 font-mono text-13 tabular-nums text-text-dim">
                {formatDuration(total.seconds)}
              </span>
            </div>

            <div className="h-2 overflow-hidden rounded-full bg-track">
              <div
                className={`h-full rounded-full ${subjectBackground(subject?.color ?? null)}`}
                // Ширина — данные, а не оформление: класса под произвольный
                // процент в Tailwind нет и быть не может.
                style={{ width: `${Math.max(total.share_percent, 2)}%` }}
              />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
