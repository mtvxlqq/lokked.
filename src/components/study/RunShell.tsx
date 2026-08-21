import type { ReactNode } from "react";

import { MODE_NAMES } from "@/components/study/modes";
import type { StudyMode } from "@/lib/tauri";

/**
 * Рамка экрана прогона: колода и режим слева, счёт и счётчик справа, под
 * ними — полоса прогресса марафона.
 *
 * Высота окна бывает меньше карточки: тогда содержимое прокручивается, а не
 * обрезается снизу.
 */
export function RunShell({
  onLeave,
  deckName,
  mode,
  progress,
  answered,
  total,
  aside,
  score,
  children,
}: {
  onLeave: () => void;
  deckName?: string;
  mode?: StudyMode;
  progress?: string;
  answered?: number;
  total?: number;
  aside?: ReactNode;
  score?: ReactNode;
  children: ReactNode;
}) {
  return (
    // Высота окна бывает меньше карточки: тогда экран должен прокручиваться,
    // а не обрезать кнопку снизу. `m-auto` на содержимом центрирует его, пока
    // место есть, и перестаёт, когда места нет.
    <div className="flex h-dvh flex-col gap-5 px-4 py-4 sm:px-8 sm:py-6">
      <header className="flex shrink-0 items-center justify-between gap-4">
        <button
          type="button"
          onClick={onLeave}
          className="min-h-11 text-13.5 text-text-dim focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          ←{" "}
          {deckName
            ? `${deckName} · ${MODE_NAMES[mode ?? "classic"]}`
            : "К карточкам"}
        </button>

        <div className="flex items-center gap-4">
          {score}
          {aside}
          {progress && (
            <span className="font-mono text-13 tabular-nums text-text-dim-2">
              {progress}
            </span>
          )}
        </div>
      </header>

      {/* Полоса прогресса — марафону: у него сотня карточек, и понимать,
          сколько осталось, надо не считая в уме. */}
      {mode === "marathon" && total ? (
        <div
          className="h-0.5 w-full shrink-0 overflow-hidden rounded-full bg-raised"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={total}
          aria-valuenow={answered ?? 0}
        >
          <div
            className="h-full bg-accent transition-[width] duration-300 ease-standard"
            style={{ width: `${((answered ?? 0) / total) * 100}%` }}
          />
        </div>
      ) : null}

      <main className="flex flex-1 flex-col overflow-y-auto">
        <div className="m-auto flex w-full max-w-2xl flex-col items-center py-2">
          {children}
        </div>
      </main>
    </div>
  );
}
