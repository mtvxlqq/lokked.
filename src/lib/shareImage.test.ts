import { describe, expect, it, vi } from "vitest";

import {
  drawShareImage,
  SHARE_HEIGHT,
  SHARE_WIDTH,
  type ShareContext,
} from "@/lib/shareImage";

/** Холст-протокол: запоминает, что на нём рисовали. */
function recorder() {
  const texts: { text: string; x: number; y: number }[] = [];
  const rects: number[][] = [];
  const styles: string[] = [];

  const context = {
    canvas: null as unknown as HTMLCanvasElement,
    fillStyle: "",
    font: "",
    textAlign: "left" as CanvasTextAlign,
    textBaseline: "top" as CanvasTextBaseline,
    shadowColor: "",
    shadowBlur: 0,
    fillRect: vi.fn((x: number, y: number, width: number, height: number) => {
      rects.push([x, y, width, height]);
      styles.push(String(context.fillStyle));
    }),
    fillText: vi.fn((text: string, x: number, y: number) => {
      texts.push({ text, x, y });
    }),
    createRadialGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
    save: vi.fn(),
    restore: vi.fn(),
  };

  return { context: context as unknown as ShareContext, texts, rects, styles };
}

const image = {
  days: 12,
  seconds: 3 * 3600 + 32 * 60 + 14,
  daysLabel: "дней подряд",
  clock: "3:32:14",
};

describe("картинка серии", () => {
  it("рисует серию, подпись, время и логотип", () => {
    const { context, texts } = recorder();

    drawShareImage(context, image);

    expect(texts.map((drawn) => drawn.text)).toEqual([
      "12",
      "ДНЕЙ ПОДРЯД",
      "3:32:14",
      "СЕГОДНЯ",
      "lokked.",
    ]);
  });

  it("ставит всё по центру холста 4:5", () => {
    const { context, texts, rects } = recorder();

    drawShareImage(context, image);

    expect(SHARE_WIDTH / SHARE_HEIGHT).toBeCloseTo(0.8);
    expect(rects[0]).toEqual([0, 0, SHARE_WIDTH, SHARE_HEIGHT]);
    for (const drawn of texts) {
      expect(drawn.x).toBe(SHARE_WIDTH / 2);
      expect(drawn.y).toBeGreaterThan(0);
      expect(drawn.y).toBeLessThan(SHARE_HEIGHT);
    }
  });

  it("кладёт цифры поверх фона, а не наоборот", () => {
    const { context, texts, rects } = recorder();

    drawShareImage(context, image);

    // Две заливки во весь холст — фон и пятно света, — и только потом текст.
    expect(rects).toHaveLength(2);
    expect(texts[0].y).toBeLessThan(texts[4].y);
  });

  it("светится вокруг главного числа и только вокруг него", () => {
    const { context } = recorder();

    drawShareImage(context, image);

    // Свечение включается и снимается: подписи под ним не размывает.
    expect(context.save).toHaveBeenCalledTimes(1);
    expect(context.restore).toHaveBeenCalledTimes(1);
  });

  it("не спотыкается о нулевую серию", () => {
    const { context, texts } = recorder();

    drawShareImage(context, { ...image, days: 0, clock: "0:00", seconds: 0 });

    expect(texts[0].text).toBe("0");
  });
});
