import { Button } from "@/components/ui";
import type { DuelView } from "@/lib/tauri";

/**
 * Передача устройства между ходами.
 *
 * Экран нужен ровно затем, чтобы спрятать предыдущий ход: пока следующий
 * игрок не сказал «готов», ни карточки, ни чужого счёта на экране нет.
 */
export function HandOver({
  view,
  busy,
  onReady,
  onLeave,
}: {
  view: DuelView;
  busy: boolean;
  onReady: () => void;
  onLeave: () => void;
}) {
  const first = view.turn === 1;

  return (
    <div className="flex min-h-dvh flex-col items-center justify-center gap-8 bg-bg-zen px-4 py-8 text-center">
      <p className="text-11 tracking-label text-text-zen-dim-2 uppercase">
        {first ? "Дуэль начинается" : "Передай устройство"}
      </p>

      <div className="flex flex-col items-center gap-3">
        <span className="font-mono text-30 leading-none tabular-nums text-text-zen-dim-2">
          {view.turn} / {view.turns}
        </span>
        <span className="text-30 font-semibold tracking-title text-text sm:text-44">
          {view.current_name}
        </span>
      </div>

      <p className="max-w-sm text-13 text-text-zen-dim-2">
        {first
          ? "Все игроки проходят одни и те же карточки в одном порядке."
          : "Результат предыдущего игрока скрыт до конца дуэли."}
      </p>

      <div className="flex flex-col items-center gap-4">
        <Button disabled={busy} onClick={onReady}>
          Я готов — начать
        </Button>
        <button
          type="button"
          onClick={onLeave}
          className="min-h-11 text-13 text-text-zen-dim-2 transition-colors duration-300 ease-standard hover:text-text-zen-dim focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          Прервать дуэль
        </button>
      </div>

      <p className="font-mono text-12.5 tracking-label text-text-zen-dim-2 uppercase">
        {view.deck_name} · {view.total} карточек · {view.seconds_per_card} с
      </p>
    </div>
  );
}
