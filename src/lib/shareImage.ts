/**
 * Картинка серии для сторис и постов: чёрный фон, светящиеся цифры и больше
 * ничего.
 *
 * Рисуется на `<canvas>` теми же цветами, что и чёрный экран, — это тот же
 * жест, только сохранённый. Функция принимает контекст, а не canvas, чтобы
 * её можно было проверить, не поднимая настоящий холст.
 */

/** Формат 4:5 — то, что не обрежут ни сторис, ни лента. */
export const SHARE_WIDTH = 1080;
export const SHARE_HEIGHT = 1350;

/**
 * Цвет из темы. На холсте классы не работают, поэтому значения берутся из
 * тех же переменных, что и везде, — руками.
 *
 * Запасное значение — имя цвета, а не второй набор токенов: оно нужно
 * только там, где темы нет вовсе (тест с поддельным холстом), и рисовать
 * нечем лучше, чем не рисовать ничего.
 */
function token(name: string, fallback: string): string {
  if (typeof globalThis.getComputedStyle !== "function") return fallback;

  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();

  return value || fallback;
}

/** Свечение вокруг главных цифр, как `--glow-streak` в теме. */
const GLOW = "rgba(226, 232, 248, 0.38)";

/** Пятно света за цифрами, акцентом темы. */
const HALO = "rgba(126, 156, 196, 0.20)";
const HALO_EDGE = "rgba(0, 0, 0, 0)";

export type ShareContext = Pick<
  CanvasRenderingContext2D,
  | "canvas"
  | "fillRect"
  | "fillText"
  | "createRadialGradient"
  | "save"
  | "restore"
> & {
  fillStyle: string | CanvasGradient | CanvasPattern;
  font: string;
  textAlign: CanvasTextAlign;
  textBaseline: CanvasTextBaseline;
  shadowColor: string;
  shadowBlur: number;
};

export type ShareImage = {
  /** Дней подряд — то самое крупное число. */
  days: number;
  /** Сколько сегодня позанимался, в секундах. */
  seconds: number;
  /** Подпись под числом: «дней подряд» с правильным окончанием. */
  daysLabel: string;
  /** Время дня в формате часов: «3:32:14». */
  clock: string;
};

/**
 * Рисует картинку в контекст размера [`SHARE_WIDTH`] × [`SHARE_HEIGHT`].
 *
 * Ничего не возвращает и ничего не знает про сохранение: превратить холст в
 * файл — дело вызывающего.
 */
export function drawShareImage(context: ShareContext, image: ShareImage): void {
  const centre = SHARE_WIDTH / 2;

  const background = token("--color-bg-zen", "black");
  const bright = token("--color-text", "white");
  const dim = token("--color-text-zen-dim", "gray");
  const faint = token("--color-text-faint", "gray");

  context.fillStyle = background;
  context.fillRect(0, 0, SHARE_WIDTH, SHARE_HEIGHT);

  // Мягкое пятно света за цифрами — то же, что делает `glow` в теме, но
  // тенью текста такой размер не берётся.
  const halo = context.createRadialGradient(
    centre,
    SHARE_HEIGHT * 0.42,
    0,
    centre,
    SHARE_HEIGHT * 0.42,
    SHARE_WIDTH * 0.62,
  );
  halo.addColorStop(0, HALO);
  halo.addColorStop(1, HALO_EDGE);
  context.fillStyle = halo;
  context.fillRect(0, 0, SHARE_WIDTH, SHARE_HEIGHT);

  context.textAlign = "center";
  context.textBaseline = "middle";

  context.save();
  context.shadowColor = GLOW;
  context.shadowBlur = 64;
  context.fillStyle = bright;
  context.font = "600 400px ui-sans-serif, system-ui, sans-serif";
  context.fillText(String(image.days), centre, SHARE_HEIGHT * 0.42);
  context.restore();

  context.fillStyle = dim;
  context.font = "500 46px ui-sans-serif, system-ui, sans-serif";
  context.fillText(image.daysLabel.toUpperCase(), centre, SHARE_HEIGHT * 0.6);

  context.fillStyle = bright;
  context.font = "500 92px ui-monospace, monospace";
  context.fillText(image.clock, centre, SHARE_HEIGHT * 0.72);

  context.fillStyle = faint;
  context.font = "500 34px ui-sans-serif, system-ui, sans-serif";
  context.fillText("СЕГОДНЯ", centre, SHARE_HEIGHT * 0.78);

  context.fillStyle = dim;
  context.font = "600 44px ui-sans-serif, system-ui, sans-serif";
  context.fillText("lokked.", centre, SHARE_HEIGHT * 0.9);
}
