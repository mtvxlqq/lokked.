/**
 * Форматирование чисел для интерфейса.
 */

/**
 * Длительность словами: «1 ч 25 мин», «25 мин», «0 мин».
 *
 * Секунды не показываем: в списке предметов важен порядок величины за день,
 * а не точность до секунды — она есть на экране таймера.
 */
export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds / 60));
  const hours = Math.floor(total / 60);
  const minutes = total % 60;

  if (hours === 0) return `${minutes} мин`;
  if (minutes === 0) return `${hours} ч`;
  return `${hours} ч ${minutes} мин`;
}

/**
 * Длительность цифрами: «25:00», «1:05:00».
 *
 * Для пресетов, где важна ровно та длительность, которую задал студент.
 */
export function formatClock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;

  const mm = hours > 0 ? String(minutes).padStart(2, "0") : String(minutes);
  const ss = String(secs).padStart(2, "0");

  return hours > 0 ? `${hours}:${mm}:${ss}` : `${mm}:${ss}`;
}

/**
 * То же время без секунд: «1:12», «0:07».
 *
 * Для чёрного экрана с настройкой «показывать только минуты»: тикающая
 * секунда — ровно то движение, ради избавления от которого этот экран и
 * существует. Часы остаются даже нулевые: `0:07` читается как время с начала
 * сессии, а «7» само по себе — нет.
 */
export function formatClockMinutes(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds / 60));
  const hours = Math.floor(total / 60);
  const minutes = total % 60;

  return `${hours}:${String(minutes).padStart(2, "0")}`;
}

/**
 * Русское склонение после числа: 1 день, 2 дня, 5 дней.
 *
 * `forms` — тройка «один / два / пять».
 */
export function plural(count: number, forms: [string, string, string]): string {
  const n = Math.abs(count) % 100;
  const last = n % 10;

  if (n > 10 && n < 20) return forms[2];
  if (last > 1 && last < 5) return forms[1];
  if (last === 1) return forms[0];
  return forms[2];
}
