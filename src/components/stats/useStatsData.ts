import { useCallback, useEffect, useState } from "react";

import { errorMessage } from "@/lib/tauri";

/** Что показывает вкладка, пока команда не ответила или если она отказала. */
export type LoadState = "loading" | "ready" | "failed";

type Result<T> = {
  /** Для чего этот ответ: период или идентификатор карточки. */
  key: string;
  data: T | null;
  error: string | null;
};

type Loaded<T> = {
  state: LoadState;
  data: T | null;
  error: string | null;
  reload: () => void;
};

/**
 * Загрузка данных вкладки: один запрос, отмена на размонтировании и повтор
 * после ошибки.
 *
 * `load` — функция от `key`, а не замыкание: она объявляется рядом с модулем
 * вкладки и не пересоздаётся на каждый рендер, поэтому эффект не запускается
 * заново без причины.
 *
 * «Загрузка…» — не состояние, а вывод: пока ответ относится к прошлому
 * `key`, вкладка ещё грузится. Так не приходится сбрасывать состояние в
 * эффекте и вызывать лишний каскад перерисовок.
 */
export function useStatsData<T>(
  load: (key: string) => Promise<T>,
  key: string,
): Loaded<T> {
  const [result, setResult] = useState<Result<T> | null>(null);
  const [reloads, setReloads] = useState(0);

  const reload = useCallback(() => {
    setResult(null);
    setReloads((count) => count + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;

    load(key)
      .then((data) => {
        if (!cancelled) setResult({ key, data, error: null });
      })
      .catch((failure: unknown) => {
        if (!cancelled) {
          setResult({ key, data: null, error: errorMessage(failure) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [load, key, reloads]);

  const fresh = result?.key === key ? result : null;

  return {
    state: fresh === null ? "loading" : fresh.error ? "failed" : "ready",
    data: fresh?.data ?? null,
    error: fresh?.error ?? null,
    reload,
  };
}
