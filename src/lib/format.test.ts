import { describe, expect, it } from "vitest";

import {
  formatClock,
  formatClockMinutes,
  formatDuration,
  plural,
} from "@/lib/format";

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

describe("formatClockMinutes", () => {
  it("прячет секунды", () => {
    expect(formatClockMinutes(72 * 60 + 24)).toBe("1:12");
  });

  it("минуты младше десяти пишет с ведущим нулём", () => {
    expect(formatClockMinutes(7 * 60 + 59)).toBe("0:07");
  });

  it("округляет вниз: минута считается пройденной, когда прошла", () => {
    expect(formatClockMinutes(59)).toBe("0:00");
    expect(formatClockMinutes(60)).toBe("0:01");
  });

  it("отрицательное время показывает нулём", () => {
    expect(formatClockMinutes(-5)).toBe("0:00");
  });
});

describe("plural", () => {
  const days: [string, string, string] = ["день", "дня", "дней"];

  it("выбирает форму по последней цифре", () => {
    expect(plural(1, days)).toBe("день");
    expect(plural(3, days)).toBe("дня");
    expect(plural(7, days)).toBe("дней");
    expect(plural(21, days)).toBe("день");
    expect(plural(102, days)).toBe("дня");
  });

  it("для второго десятка всегда «дней»", () => {
    expect(plural(11, days)).toBe("дней");
    expect(plural(12, days)).toBe("дней");
    expect(plural(114, days)).toBe("дней");
  });

  it("ноль — тоже «дней»", () => {
    expect(plural(0, days)).toBe("дней");
  });
});
