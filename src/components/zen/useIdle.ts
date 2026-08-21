import { useEffect, useState } from "react";

/** Сколько экран ждёт без единого движения, прежде чем погаснуть. */
export const IDLE_MS = 5000;

/**
 * `true`, когда с последнего действия прошло `delayMs`.
 *
 * Слушает окно, а не свой узел: погаснуть экран должен и тогда, когда мышь
 * замерла за пределами цифр, а вернуться — от любого движения, откуда бы оно
 * ни пришло.
 */
export function useIdle(delayMs: number = IDLE_MS): boolean {
  const [idle, setIdle] = useState(false);

  useEffect(() => {
    let timer = 0;

    function wake() {
      setIdle(false);
      window.clearTimeout(timer);
      timer = window.setTimeout(() => setIdle(true), delayMs);
    }

    // `click` в списке не ради мыши, а ради ассистивных технологий и
    // клавиатуры: нажатие, пришедшее не от указателя, тоже действие.
    const events = [
      "mousemove",
      "mousedown",
      "keydown",
      "wheel",
      "touchstart",
      "touchmove",
      "click",
    ] as const;

    for (const event of events) {
      window.addEventListener(event, wake, { passive: true });
    }
    wake();

    return () => {
      window.clearTimeout(timer);
      for (const event of events) {
        window.removeEventListener(event, wake);
      }
    };
  }, [delayMs]);

  return idle;
}
