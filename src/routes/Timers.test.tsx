import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Timers } from "@/routes/Timers";
import type { Preset, Subject, TodayTotals } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const algebra: Subject = {
  id: "s-1",
  name: "Алгебра",
  color: "subject-1",
  icon: null,
  position: 0,
};
const physics: Subject = {
  id: "s-2",
  name: "Физика",
  color: "subject-2",
  icon: null,
  position: 1,
};

const preset: Preset = {
  id: "p-1",
  subject_id: "s-1",
  name: "Классический",
  mode: "pomodoro",
  work_seconds: 25 * 60,
  break_seconds: 5 * 60,
  long_break_seconds: 15 * 60,
  cycles_before_long: 4,
  auto_start_next: false,
  is_default: true,
};

/** Пустой день: ничего не изучено, серии нет, граница — ближайшая полночь. */
const EMPTY_DAY: TodayTotals = {
  day_key: "2026-08-21",
  seconds_by_subject: [],
  total_seconds: 0,
  pomodoros: 0,
  streak_days: 0,
  next_boundary: "2026-08-22T00:00:00Z",
};

/**
 * Отвечает на три команды, которыми экран грузится. Задаётся тем, что должно
 * лежать в базе, — сами вызовы `invoke` тест не перечисляет.
 */
function backend(data: {
  subjects?: Subject[];
  presets?: Preset[];
  totals?: Partial<TodayTotals>;
}) {
  invoke.mockImplementation((command: string) => {
    switch (command) {
      case "list_subjects":
        return Promise.resolve(data.subjects ?? []);
      case "list_presets":
        return Promise.resolve(data.presets ?? []);
      case "today_totals":
        return Promise.resolve({ ...EMPTY_DAY, ...data.totals });
      default:
        return Promise.reject(new Error(`неожиданная команда: ${command}`));
    }
  });
}

function renderScreen() {
  const router = createMemoryRouter(
    [
      { path: "/", element: <Timers /> },
      { path: "/timer/:subjectId", element: <p>Экран сессии</p> },
    ],
    { initialEntries: ["/"] },
  );

  render(<RouterProvider router={router} />);
  return router;
}

describe("Timers", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("предлагает добавить первый предмет, когда их нет", async () => {
    backend({});
    renderScreen();

    expect(await screen.findByText("Предметов пока нет")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Добавить предмет" }),
    ).toBeInTheDocument();
  });

  it("показывает предметы с временем за сегодня", async () => {
    backend({
      subjects: [algebra, physics],
      totals: { seconds_by_subject: [["s-1", 3600 + 12 * 60]] },
    });
    renderScreen();

    expect(await screen.findByText("Алгебра")).toBeInTheDocument();
    expect(screen.getByText("1 ч 12 мин")).toBeInTheDocument();
    // У предмета без записей за сегодня — честный ноль, а не пустая ячейка.
    expect(screen.getByText("0 мин")).toBeInTheDocument();
  });

  it("«Старт» ведёт на экран сессии этого предмета", async () => {
    const user = userEvent.setup();
    backend({ subjects: [algebra] });
    const router = renderScreen();

    await screen.findByText("Алгебра");
    await user.click(screen.getByRole("button", { name: "Старт" }));

    expect(await screen.findByText("Экран сессии")).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/timer/s-1");
  });

  it("показывает пресеты с режимом и привязкой", async () => {
    backend({ subjects: [algebra], presets: [preset] });
    renderScreen();

    expect(await screen.findByText("Классический")).toBeInTheDocument();
    expect(
      screen.getByText("Помодоро · 25:00 · 5:00 · 15:00 × 4 · Алгебра"),
    ).toBeInTheDocument();
    expect(screen.getByText("По умолчанию")).toBeInTheDocument();
  });

  it("перечитывает данные после сохранения в диалоге", async () => {
    const user = userEvent.setup();
    backend({ subjects: [algebra] });
    renderScreen();

    await screen.findByText("Алгебра");
    await user.click(screen.getByRole("button", { name: "Новый предмет" }));

    invoke.mockClear();
    // Диалог создаёт предмет и сообщает экрану, что список устарел.
    invoke.mockImplementationOnce(() => Promise.resolve(physics));
    backend({ subjects: [algebra, physics] });

    await user.type(screen.getByLabelText("Название"), "Физика");
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(await screen.findByText("Физика")).toBeInTheDocument();
  });

  it("сообщает об отказе команды и даёт повторить", async () => {
    const user = userEvent.setup();
    invoke.mockRejectedValue({
      kind: "database",
      message: "database query failed: disk I/O error",
    });
    renderScreen();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "disk I/O error",
    );

    backend({ subjects: [algebra] });
    await user.click(screen.getByRole("button", { name: "Повторить" }));

    expect(await screen.findByText("Алгебра")).toBeInTheDocument();
  });

  it("показывает сводку за учебный день", async () => {
    backend({
      subjects: [algebra],
      totals: {
        seconds_by_subject: [["s-1", 3 * 3600 + 32 * 60 + 14]],
        total_seconds: 3 * 3600 + 32 * 60 + 14,
        pomodoros: 6,
        streak_days: 12,
      },
    });
    renderScreen();

    expect(await screen.findByText("3:32:14")).toBeInTheDocument();
    expect(screen.getByText("6")).toBeInTheDocument();
    expect(screen.getByText("12 дней")).toBeInTheDocument();
  });

  it("склоняет дни серии", async () => {
    backend({ subjects: [algebra], totals: { streak_days: 1 } });
    renderScreen();

    expect(await screen.findByText("1 день")).toBeInTheDocument();
  });

  it("перечитывает сводку, когда наступает граница учебного дня", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      backend({
        subjects: [algebra],
        totals: {
          total_seconds: 40 * 60,
          streak_days: 3,
          next_boundary: new Date(Date.now() + 5000).toISOString(),
        },
      });
      renderScreen();
      await screen.findByText("40:00");

      // Новый день: время обнулилось, серия осталась — её ведёт база, а не экран.
      backend({
        subjects: [algebra],
        totals: {
          day_key: "2026-08-22",
          total_seconds: 0,
          streak_days: 3,
          next_boundary: new Date(Date.now() + 86_400_000).toISOString(),
        },
      });

      await act(async () => {
        vi.advanceTimersByTime(6000);
      });

      expect(await screen.findByText("0:00")).toBeInTheDocument();
      expect(screen.getByText("3 дня")).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("перечитывает сводку при возврате из фона", async () => {
    backend({ subjects: [algebra], totals: { total_seconds: 40 * 60 } });
    renderScreen();
    await screen.findByText("40:00");

    backend({ subjects: [algebra], totals: { total_seconds: 55 * 60 } });
    await act(async () => {
      document.dispatchEvent(new Event("visibilitychange"));
    });

    expect(await screen.findByText("55:00")).toBeInTheDocument();
  });
});
