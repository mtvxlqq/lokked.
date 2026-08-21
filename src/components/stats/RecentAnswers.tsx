import { cn } from "@/lib/cn";
import type { Grade } from "@/lib/tauri";

/** Подписи оценок — те же, что и на кнопках под карточкой. */
const LABELS: Record<Grade, string> = {
  again: "Не помню",
  hard: "С трудом",
  good: "Знаю",
  easy: "Легко",
};

/**
 * Последние ответы цепочкой точек, слева направо от старых к новым.
 *
 * Цвет — только подсказка: у каждой точки есть подпись для скринридера и
 * всплывающая для мыши, а промах отличается от попадания ещё и размером
 * заливки.
 */
export function RecentAnswers({ grades }: { grades: Grade[] }) {
  if (grades.length === 0) {
    return (
      <p className="text-14 text-text-dim">
        Эту карточку ещё ни разу не показывали.
      </p>
    );
  }

  return (
    <ol className="flex flex-wrap items-center gap-2">
      {grades.map((grade, index) => (
        <li
          // Ответы неразличимы между собой, и порядок — единственное, что их
          // отличает: индекс здесь и есть идентичность.
          key={index}
          title={LABELS[grade]}
          className={cn(
            "size-3.5 rounded-full",
            grade === "again" ? "bg-danger" : "bg-accent-alt-teal",
          )}
        >
          <span className="sr-only">{LABELS[grade]}</span>
        </li>
      ))}
    </ol>
  );
}
