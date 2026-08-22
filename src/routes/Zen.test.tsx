import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Zen } from "@/routes/Zen";
import type { SessionSnapshot, ZenSettings } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const setFullscreen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setFullscreen }),
}));

const running: SessionSnapshot = {
  subject_id: "s-1",
  subject_name: "Математический анализ",
  subject_color: "subject-1",
  preset_id: null,
  mode: "countup",
  phase: "work",
  status: "running",
  cycle: 1,
  cycles_before_long: null,
  elapsed_seconds: 12 * 60 + 24,
  session_seconds: 72 * 60 + 24,
  remaining_seconds: null,
  target_seconds: null,
  phase_finished: false,
  interruptions: 0,
  auto_start_next: false,
};

/** Отвечает на команды чёрного экрана; снимок можно менять между опросами. */
function backend(
  options: {
    snapshot?: SessionSnapshot | null;
    settings?: Partial<ZenSettings>;
  } = {},
) {
  let snapshot = options.snapshot === undefined ? running : options.snapshot;

  invoke.mockImplementation((command: string) => {
    switch (command) {
      case "session_snapshot":
        return Promise.resolve(snapshot);
      case "zen_settings":
        return Promise.resolve({
          minutes_only: false,
          font_size: "normal",
          dim_when_idle: true,
          ...options.settings,
        } satisfies ZenSettings);
      case "pause_session":
        snapshot = { ...(snapshot ?? running), status: "paused" };
        return Promise.resolve(snapshot);
      case "resume_session":
        snapshot = { ...(snapshot ?? running), status: "running" };
        return Promise.resolve(snapshot);
      case "stop_session":
        snapshot = null;
        return Promise.resolve(null);
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

/** Отдельный маршрут для экрана сессии — на него ведёт выход из Zen. */
function renderZen() {
  const router = createMemoryRouter(
    [
      { path: "/zen", element: <Zen /> },
      { path: "/timer/:subjectId", element: <p>экран сессии</p> },
      { path: "/", element: <p>список предметов</p> },
    ],
    { initialEntries: ["/zen"] },
  );

  const { container } = render(<RouterProvider router={router} />);
  // Корневой узел экрана: на нём висят жесты, роли у него нет.
  return { router, surface: () => container.firstElementChild! };
}

beforeEach(() => {
  invoke.mockReset();
  setFullscreen.mockReset();
  setFullscreen.mockResolvedValue(undefined);
});

afterEach(() => {
  vi.useRealTimers();
});

describe("чёрный экран", () => {
  it("показывает время с начала сессии и предмет", async () => {
    backend();
    renderZen();

    expect(await screen.findByText("1:12:24")).toBeInTheDocument();
    expect(screen.getByText("Математический анализ")).toBeInTheDocument();
  });

  it("с настройкой «только минуты» прячет секунды", async () => {
    backend({ settings: { minutes_only: true } });
    renderZen();

    expect(await screen.findByText("1:12")).toBeInTheDocument();
    expect(screen.queryByText("1:12:24")).not.toBeInTheDocument();
  });

  it("разворачивает окно на весь экран и возвращает его при выходе", async () => {
    backend();
    const { router } = renderZen();
    await screen.findByText("1:12:24");

    expect(setFullscreen).toHaveBeenCalledWith(true);

    await act(async () => {
      await router.navigate("/");
    });

    await waitFor(() => expect(setFullscreen).toHaveBeenCalledWith(false));
  });

  it("по Esc возвращает к экрану сессии", async () => {
    backend();
    renderZen();
    await screen.findByText("1:12:24");

    await userEvent.keyboard("{Escape}");

    expect(await screen.findByText("экран сессии")).toBeInTheDocument();
  });

  it("по пробелу ставит на паузу и снимает с неё", async () => {
    backend();
    renderZen();
    await screen.findByText("1:12:24");

    await userEvent.keyboard(" ");
    expect(await screen.findByText("Пауза")).toBeInTheDocument();

    await userEvent.keyboard(" ");
    await waitFor(() =>
      expect(screen.queryByText("Пауза")).not.toBeInTheDocument(),
    );
  });

  it("по Q завершает сессию и уходит к списку предметов", async () => {
    backend();
    renderZen();
    await screen.findByText("1:12:24");

    await userEvent.keyboard("q");

    expect(await screen.findByText("список предметов")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("stop_session");
  });

  it("работает и на другой раскладке: важна клавиша, а не буква", async () => {
    backend();
    renderZen();
    await screen.findByText("1:12:24");

    // Русская «й» — это та же физическая клавиша KeyQ.
    await userEvent.keyboard("[KeyQ]");

    expect(await screen.findByText("список предметов")).toBeInTheDocument();
  });

  it("без сессии на экране делать нечего", async () => {
    backend({ snapshot: null });
    renderZen();

    expect(await screen.findByText("список предметов")).toBeInTheDocument();
  });

  it("гаснет через пять секунд бездействия и возвращается от движения", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    backend();
    renderZen();

    const digits = await screen.findByText("1:12:24");
    expect(digits.className).toContain("glow-timer");
    expect(digits.className).not.toContain("glow-timer-dim");

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });
    expect(digits.className).toContain("glow-timer-dim");

    await act(async () => {
      window.dispatchEvent(new MouseEvent("mousemove"));
    });
    expect(digits.className).not.toContain("glow-timer-dim");
  });

  it("с выключенным затемнением экран не гаснет", async () => {
    // Таймер в углу зрения читают, не касаясь мыши: кто выключил затемнение,
    // тот именно этого и хотел.
    vi.useFakeTimers({ shouldAdvanceTime: true });
    backend({ settings: { dim_when_idle: false } });
    renderZen();

    const digits = await screen.findByText("1:12:24");

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(digits.className).toContain("glow-timer");
    expect(digits.className).not.toContain("glow-timer-dim");
  });

  it("первое касание погасшего экрана только возвращает свет", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    backend();
    renderZen();
    const digits = await screen.findByText("1:12:24");

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    const surface = screen.getByRole("button", { name: "Пауза" });
    await act(async () => {
      surface.click();
    });

    // Экран проснулся, но сессия идёт дальше.
    expect(digits.className).not.toContain("glow-timer-dim");
    expect(screen.queryByText("Пауза")).not.toBeInTheDocument();
    expect(invoke).not.toHaveBeenCalledWith("pause_session");
  });

  it("свайп вниз возвращает к экрану сессии", async () => {
    backend();
    const { surface } = renderZen();
    await screen.findByText("1:12:24");

    fireEvent.touchStart(surface(), { touches: [{ clientY: 120 }] });
    fireEvent.touchEnd(surface(), { changedTouches: [{ clientY: 300 }] });

    expect(await screen.findByText("экран сессии")).toBeInTheDocument();
  });

  it("короткое касание за свайп не считается", async () => {
    backend();
    const { surface } = renderZen();
    await screen.findByText("1:12:24");

    fireEvent.touchStart(surface(), { touches: [{ clientY: 120 }] });
    fireEvent.touchEnd(surface(), { changedTouches: [{ clientY: 150 }] });

    expect(screen.queryByText("экран сессии")).not.toBeInTheDocument();
  });

  it("касание по центру на живом экране ставит паузу", async () => {
    backend();
    renderZen();
    await screen.findByText("1:12:24");

    await userEvent.click(screen.getByRole("button", { name: "Пауза" }));

    expect(await screen.findByText("Пауза")).toBeInTheDocument();
  });
});
