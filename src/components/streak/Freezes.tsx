import { cn } from "@/lib/cn";
import { plural } from "@/lib/format";

/**
 * Заморозки: сколько пропусков серия переживёт, не оборвавшись.
 *
 * Показаны штучно, а не числом: три ячейки видно одним взглядом, а «2 / 3»
 * рядом — на случай, когда запас вырастет.
 */
export function Freezes({
  freezes,
  max,
  every,
}: {
  freezes: number;
  max: number;
  every: number;
}) {
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <ul className="flex gap-2" aria-hidden="true">
          {Array.from({ length: max }, (_, index) => (
            <li
              key={index}
              className={cn(
                "size-9 rounded-lg border sm:size-11",
                index < freezes
                  ? "border-transparent bg-streak-frozen"
                  : "border-border-strong bg-raised",
              )}
            />
          ))}
        </ul>
        <span className="text-15.5 tabular-nums text-text-1">
          {freezes} / {max}
        </span>
      </div>

      <p className="text-12.5 text-text-dim">
        Пропущенный день закрывается заморозкой, и серия продолжается. Одна
        начисляется за каждые {every} {plural(every, ["день", "дня", "дней"])}{" "}
        подряд.
      </p>
    </div>
  );
}
