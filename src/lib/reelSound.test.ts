import { describe, expect, it } from "vitest";

import { tickTimes } from "@/lib/reelSound";

const SPIN_MS = 1300;

describe("щелчки барабана", () => {
  it("щёлкает не чаще, чем строк проехало мимо", () => {
    const times = tickTimes(22, SPIN_MS);

    expect(times.length).toBeGreaterThan(0);
    expect(times.length).toBeLessThanOrEqual(22);
  });

  it("укладывается в прокрут и идёт по возрастанию", () => {
    const times = tickTimes(22, SPIN_MS);

    expect(times[0]).toBeGreaterThan(0);
    expect(times[times.length - 1]).toBeLessThanOrEqual(SPIN_MS);

    for (let index = 1; index < times.length; index += 1) {
      expect(times[index]).toBeGreaterThan(times[index - 1]);
    }
  });

  it("замедляется к концу — как и сама лента", () => {
    // Смысл звука в этом: барабан слышно останавливающимся, а не просто
    // трещащим. Первый промежуток должен быть заметно короче последнего.
    const times = tickTimes(22, SPIN_MS);

    const first = times[1] - times[0];
    const last = times[times.length - 1] - times[times.length - 2];

    expect(last).toBeGreaterThan(first * 3);
  });

  it("не ставит два щелчка подряд ближе слышимого промежутка", () => {
    // В начале лента идёт так быстро, что строки мелькают чаще, чем ухо
    // различает щелчки: слитный треск — это шум, а не барабан.
    const gap = 30;
    const times = tickTimes(22, SPIN_MS, gap);

    for (let index = 1; index < times.length; index += 1) {
      expect(times[index] - times[index - 1]).toBeGreaterThanOrEqual(gap);
    }
  });

  it("на пустой прокрут щелчков нет", () => {
    expect(tickTimes(0, SPIN_MS)).toEqual([]);
  });
});
