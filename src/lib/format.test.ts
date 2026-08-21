import { describe, expect, it } from "vitest";

import { formatClock, formatDuration } from "@/lib/format";

describe("formatDuration", () => {
  it("показывает нулевое время, а не пустую строку", () => {
    expect(formatDuration(0)).toBe("0 мин");
  });

  it("округляет вниз до минуты", () => {
    expect(formatDuration(59)).toBe("0 мин");
    expect(formatDuration(119)).toBe("1 мин");
  });

  it("часы без минут пишет без хвоста", () => {
    expect(formatDuration(3600)).toBe("1 ч");
  });

  it("часы с минутами пишет полностью", () => {
    expect(formatDuration(3600 + 25 * 60)).toBe("1 ч 25 мин");
  });

  it("отрицательное время считает нулём", () => {
    expect(formatDuration(-60)).toBe("0 мин");
  });
});

describe("formatClock", () => {
  it("до часа показывает минуты и секунды", () => {
    expect(formatClock(25 * 60)).toBe("25:00");
    expect(formatClock(65)).toBe("1:05");
  });

  it("от часа добавляет часы и дополняет минуты нулём", () => {
    expect(formatClock(3600 + 5 * 60)).toBe("1:05:00");
  });
});
