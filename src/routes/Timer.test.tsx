import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Timer } from "@/routes/Timer";
import type { SessionSnapshot } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

// Уведомления и звук — побочный эффект смены фазы; в тестах достаточно знать,
// что экран не падает без системного уведомителя.
vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: () => Promise.resolve(true),
  requestPermission: () => Promise.resolve("granted"),
  sendNotification: vi.fn(),
}));

const running: SessionSnapshot = {
  subject_id: "s-1",
  subject_name: "Алгебра",
  subject_color: "subject-1",
  preset_id: "p-1",
  mode: "pomodoro",
  phase: "work",
  status: "running",
  cycle: 2,
  cycles_before_long: 4,
  elapsed_seconds: 5 * 60,
  session_seconds: 30 * 60,
  remaining_seconds: 20 * 60,
  target_seconds: 25 * 60,
  phase_finished: false,
  interruptions: 0,
  auto_start_next: false,
};

const stopwatch: SessionSnapshot = {
  ...running,
  preset_id: null,
  mode: "countup",
  cycle: 1,
  cycles_before_long: null,
  elapsed_seconds: 90,
  session_seconds: 90,
  remaining_seconds: null,
  target_seconds: null,
};

/**
 * Отвечает на команды сессии. `snapshot` — то, что видно на экране; можно
 * подменять между тиками, имитируя ход времени.
 */
function backend(options: {
  snapshot?: SessionSnapshot | null;
  onStart?: SessionSnapshot;
  awaySeconds?: number;
}) {
  let snapshot = options.snapshot ?? null;

  invoke.mockImplementation((command: string) => {
    switch (command) {
      case "session_snapshot":
        return Promise.resolve(snapshot);
      case "start_session":
        snapshot = options.onStart ?? running;
        return Promise.resolve(snapshot);
      case "pause_session":
        snapshot = { ...(snapshot ?? running), status: "paused" };
        return Promise.resolve(snapshot);
      case "resume_session":
        snapshot = { ...(snapshot ?? running), status: "running" };
        return Promise.resolve(snapshot);
      case "session_mark_interruption":
        snapshot = {
          ...(snapshot ?? running),
          interruptions: (snapshot ?? running).interruptions + 1,
        };
        return Promise.resolve(snapshot);
      case "session_skip_phase":
        snapshot = { ...(snapshot ?? running), phase: "break" };
        return Promise.resolve(snapshot);
      case "stop_session":
        snapshot = null;
        return Promise.resolve(null);
      case "session_report_return":
        return Promise.resolve({
          away_seconds: options.awaySeconds ?? 0,
          needs_decision: (options.awaySeconds ?? 0) >= 600,
        });
      case "session_discard_away":
        snapshot = { ...(snapshot ?? running), elapsed_seconds: 60 };
        return Promise.resolve(snapshot);
      default:
        return Promise.reject(new Error(`неожиданная команда: ${command}`));
    }
  });

  return {
    set(next: SessionSnapshot | null) {
      snapshot = next;
    },
  };
}

function renderScreen(subjectId = "s-1") {
  const router = createMemoryRouter(
    [
      { path: "/", element: <p>Список предметов</p> },
      { path: "/timer/:subjectId", element: <Timer /> },
      { path: "/zen", element: <p>Чёрный экран</p> },
    ],
    { initialEntries: [`/timer/${subjectId}`] },
  );

  render(<RouterProvider router={router} />);
  return router;
}

/** Аргументы, с которыми вызывали команду. */
function calls(command: string) {
  return invoke.mock.calls.filter(([name]) => name === command);
}

describe("Timer", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("начинает сессию, если ни одна не идёт", async () => {
    backend({ snapshot: null, onStart: running });
    renderScreen();

    expect(await screen.findByText("Алгебра")).toBeInTheDocument();
    expect(calls("start_session")[0]?.[1]).toEqual({ subjectId: "s-1" });
  });

  it("показывает уже идущую сессию, не начиная новую", async () => {
    backend({ snapshot: running });
    renderScreen();

    await screen.findByText("Алгебра");
    expect(calls("start_session")).toHaveLength(0);
  });

  it("уводит к сессии, которая идёт по другому предмету", async () => {
    backend({ snapshot: { ...running, subject_id: "s-2" } });
    const router = renderScreen("s-1");

    await waitFor(() =>
      expect(router.state.location.pathname).toBe("/timer/s-2"),
    );
    expect(calls("start_session")).toHaveLength(0);
  });

  it("показывает остаток фазы и подпись «работа 2/4»", async () => {
    backend({ snapshot: running });
    renderScreen();

    // Помодоро идёт вниз: на экране остаток, а не набранное.
    expect(await screen.findByText("20:00")).toBeInTheDocument();
    expect(screen.getByText("Работа 2/4")).toBeInTheDocument();
  });

  it("у секундомера показывает набранное время без номера цикла", async () => {
    backend({ snapshot: stopwatch });
    renderScreen();

    expect(await screen.findByText("1:30")).toBeInTheDocument();
    expect(screen.getByText("Работа")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "На перерыв" }),
    ).not.toBeInTheDocument();
  });

  it("перечитывает состояние на каждом тике", async () => {
    const control = backend({ snapshot: running });
    renderScreen();

    await screen.findByText("20:00");
    control.set({
      ...running,
      elapsed_seconds: 6 * 60,
      remaining_seconds: 19 * 60,
    });

    expect(await screen.findByText("19:00")).toBeInTheDocument();
  });

  it("ставит на паузу и снимает с неё", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    renderScreen();

    await user.click(await screen.findByRole("button", { name: "Пауза" }));

    expect(calls("pause_session")).toHaveLength(1);
    const resume = await screen.findByRole("button", { name: "Продолжить" });

    await user.click(resume);
    expect(calls("resume_session")).toHaveLength(1);
  });

  it("отмечает, что студент отвлёкся, не трогая таймер", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    renderScreen();

    await user.click(await screen.findByRole("button", { name: "Отвлёкся" }));

    expect(calls("session_mark_interruption")).toHaveLength(1);
    expect(await screen.findByText("Отвлекался: 1")).toBeInTheDocument();
  });

  it("переводит помодоро на перерыв", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    renderScreen();

    await user.click(await screen.findByRole("button", { name: "На перерыв" }));

    expect(calls("session_skip_phase")).toHaveLength(1);
    expect(await screen.findByText("Перерыв")).toBeInTheDocument();
  });

  it("по «Стопу» завершает сессию и возвращает к списку", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    const router = renderScreen();

    await user.click(await screen.findByRole("button", { name: "Стоп" }));

    expect(calls("stop_session")).toHaveLength(1);
    await waitFor(() => expect(router.state.location.pathname).toBe("/"));
  });

  it("возвращает к списку, если сессию завершили где-то ещё", async () => {
    const control = backend({ snapshot: running });
    const router = renderScreen();

    await screen.findByText("Алгебра");
    control.set(null);

    await waitFor(() => expect(router.state.location.pathname).toBe("/"));
  });

  it("уводит на чёрный экран", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    const router = renderScreen();

    await user.click(
      await screen.findByRole("button", { name: "Чёрный экран" }),
    );

    await waitFor(() => expect(router.state.location.pathname).toBe("/zen"));
  });

  it("сообщает об отказе команды", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running });
    renderScreen();

    const pause = await screen.findByRole("button", { name: "Пауза" });
    invoke.mockImplementationOnce(() =>
      Promise.reject({ kind: "conflict", message: "сессия уже на паузе" }),
    );
    await user.click(pause);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "сессия уже на паузе",
    );
  });

  it("после долгого отсутствия спрашивает про время и отбрасывает его", async () => {
    const user = userEvent.setup();
    backend({ snapshot: running, awaySeconds: 3600 });
    renderScreen();

    await screen.findByText("Алгебра");
    document.dispatchEvent(new Event("visibilitychange"));

    expect(await screen.findByText("Тебя не было")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Отбросить" }));

    expect(calls("session_discard_away")).toHaveLength(1);
    await waitFor(() =>
      expect(screen.queryByText("Тебя не было")).not.toBeInTheDocument(),
    );
  });

  it("после короткого отсутствия ничего не спрашивает", async () => {
    backend({ snapshot: running, awaySeconds: 120 });
    renderScreen();

    await screen.findByText("Алгебра");
    document.dispatchEvent(new Event("visibilitychange"));

    await waitFor(() => expect(calls("session_report_return")).toHaveLength(1));
    expect(screen.queryByText("Тебя не было")).not.toBeInTheDocument();
  });
});
