import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Streak } from "@/routes/Streak";
import type { StreakDay, StreakView } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

/** Август 2026: дни до 21-го зачтены, 21-е идёт, остальное впереди. */
function august(): StreakDay[] {
  return Array.from({ length: 31 }, (_, index) => {
    const day = `2026-08-${String(index + 1).padStart(2, "0")}`;

    if (index + 1 < 10) return { day, seconds: 0, state: "missed" as const };
    if (index + 1 === 15) return { day, seconds: 0, state: "frozen" as const };
    if (index + 1 <= 21)
      return { day, seconds: 45 * 60, state: "counted" as const };
    return { day, seconds: 0, state: "future" as const };
  });
}

const view: StreakView = {
  today: "2026-08-21",
  today_seconds: 3 * 3600 + 32 * 60,
  min_seconds: 600,
  day_start_seconds: 4 * 3600,
  current: 12,
  longest: 27,
  longest_from: "2026-03-15",
  longest_to: "2026-04-10",
  freezes: 2,
  max_freezes: 3,
  freeze_every: 10,
  frozen_days: 1,
  milestones: [
    { target: 7, reached: true, reached_on: "2026-07-30", remaining: 0 },
    { target: 30, reached: false, reached_on: null, remaining: 18 },
    { target: 100, reached: false, reached_on: null, remaining: 88 },
  ],
  month: { year: 2026, month: 8, days: august() },
};

function backend(options: { page?: StreakView; fails?: boolean } = {}) {
  invoke.mockImplementation((command: string) => {
    if (command === "streak_view") {
      return options.fails
        ? Promise.reject({ kind: "database", message: "база недоступна" })
        : Promise.resolve(options.page ?? view);
    }
    if (command === "streak_save_image") {
      return Promise.resolve("/home/студент/Изображения/lokked-streak.png");
    }
    return Promise.reject(new Error(`неожиданная команда ${command}`));
  });
}

/**
 * jsdom не умеет рисовать: холст подменяется заглушкой, иначе диалог
 * «поделиться» не доходит до сохранения. Что именно рисуется, проверяет
 * `shareImage.test.ts` — здесь важно, что нарисованное уходит в команду.
 */
beforeEach(() => {
  invoke.mockReset();

  HTMLCanvasElement.prototype.getContext = vi.fn(
    () =>
      new Proxy(
        {},
        {
          get: () => vi.fn(() => ({ addColorStop: vi.fn() })),
          set: () => true,
        },
      ),
  ) as unknown as HTMLCanvasElement["getContext"];
  HTMLCanvasElement.prototype.toDataURL = vi.fn(
    () => "data:image/png;base64,iVBORw0KGgo=",
  );
});

describe("страница серии", () => {
  /** Карточка страницы по её заголовку. */
  function panel(name: string) {
    return screen.getByRole("group", { name });
  }

  it("показывает серию, сегодняшний минимум и рекорд", async () => {
    backend();
    render(<Streak />);

    const current = within(
      await screen.findByRole("group", {
        name: "Текущая серия",
      }),
    );
    expect(current.getByText("12")).toBeInTheDocument();
    expect(current.getByText("дней подряд")).toBeInTheDocument();
    expect(
      current.getByText("Сегодня зачтено — 3 ч 32 мин, минимум 10 мин."),
    ).toBeInTheDocument();

    const record = within(panel("Рекорд"));
    expect(record.getByText("27")).toBeInTheDocument();
    expect(record.getByText("15 марта — 10 апреля")).toBeInTheDocument();
  });

  it("рисует календарь месяца с границей дня", async () => {
    backend();
    render(<Streak />);

    expect(await screen.findByText("Август 2026")).toBeInTheDocument();
    expect(screen.getByText("граница дня — 04:00")).toBeInTheDocument();
    // Сегодняшний день помечен как текущий, а замороженный — как заморозка.
    expect(screen.getByText("21")).toHaveAttribute("aria-current", "date");
    expect(screen.getByText("15")).toHaveAttribute(
      "title",
      expect.stringContaining("заморозка"),
    );
  });

  it("показывает запас заморозок", async () => {
    backend();
    render(<Streak />);

    expect(await screen.findByText("2 / 3")).toBeInTheDocument();
    expect(
      screen.getByText(/Одна начисляется за каждые 10 дней подряд/),
    ).toBeInTheDocument();
  });

  it("показывает взятую веху и то, сколько осталось до следующей", async () => {
    backend();
    render(<Streak />);

    expect(await screen.findByText("взято 30 июля")).toBeInTheDocument();
    expect(screen.getByText("осталось 18 дней")).toBeInTheDocument();
    expect(screen.getByText("осталось 88 дней")).toBeInTheDocument();

    const toThirty = screen.getByRole("progressbar", {
      name: "30 дней подряд",
    });
    expect(toThirty).toHaveAttribute("aria-valuenow", "12");
  });

  it("сохраняет картинку серии и показывает, куда", async () => {
    backend();
    render(<Streak />);

    await userEvent.click(
      await screen.findByRole("button", { name: "Поделиться серией" }),
    );

    const dialog = screen.getByRole("dialog");
    await userEvent.click(
      within(dialog).getByRole("button", { name: "Сохранить" }),
    );

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "streak_save_image",
        expect.objectContaining({ png: expect.any(String) }),
      ),
    );
    expect(
      await screen.findByText("/home/студент/Изображения/lokked-streak.png"),
    ).toBeInTheDocument();
  });

  it("предлагает повторить, если серия не прочиталась", async () => {
    backend({ fails: true });
    render(<Streak />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "база недоступна",
    );

    invoke.mockReset();
    backend();
    await userEvent.click(screen.getByRole("button", { name: "Повторить" }));

    expect(
      await screen.findByRole("group", { name: "Текущая серия" }),
    ).toBeInTheDocument();
  });

  it("не врёт про серию, которой нет", async () => {
    backend({
      page: {
        ...view,
        current: 0,
        longest: 0,
        longest_from: null,
        longest_to: null,
        freezes: 0,
        today_seconds: 0,
        milestones: [
          { target: 7, reached: false, reached_on: null, remaining: 7 },
          { target: 30, reached: false, reached_on: null, remaining: 30 },
          { target: 100, reached: false, reached_on: null, remaining: 100 },
        ],
      },
    });
    render(<Streak />);

    const current = within(
      await screen.findByRole("group", { name: "Текущая серия" }),
    );
    expect(current.getByText("0")).toBeInTheDocument();
    expect(current.getByText("дней подряд")).toBeInTheDocument();
    expect(within(panel("Заморозки")).getByText("0 / 3")).toBeInTheDocument();
    expect(screen.getByText("осталось 7 дней")).toBeInTheDocument();
  });
});
