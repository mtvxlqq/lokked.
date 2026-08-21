import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { DesktopEvents } from "@/components/DesktopEvents";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

/** Подписки, которые компонент завёл: имя события → обработчик. */
const listeners = vi.hoisted(() => new Map<string, (event: unknown) => void>());
const unlisten = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (event: unknown) => void) => {
    listeners.set(name, handler);
    return Promise.resolve(unlisten);
  },
}));

/** Рендерит слушателя с парой экранов под ним. */
function renderApp() {
  const router = createMemoryRouter(
    [
      {
        element: <DesktopEvents />,
        children: [
          { path: "/", element: <p>Таймеры</p> },
          { path: "zen", element: <p>Чёрный экран</p> },
        ],
      },
    ],
    { initialEntries: ["/"] },
  );

  const { unmount } = render(<RouterProvider router={router} />);

  return { router, unmount };
}

/** Присылает событие из Rust, как это делает `app.emit`. */
function emit(name: string, payload?: unknown) {
  const handler = listeners.get(name);
  if (!handler) throw new Error(`никто не слушает ${name}`);

  handler({ payload });
}

beforeEach(() => {
  listeners.clear();
  unlisten.mockClear();
  invoke.mockReset();
  invoke.mockImplementation((command: string) => {
    switch (command) {
      case "cli_pending_zen":
        return Promise.resolve(false);
      case "resume_session":
        return Promise.resolve(null);
      default:
        return Promise.reject(new Error(`неожиданная команда: ${command}`));
    }
  });
});

describe("горячая клавиша", () => {
  it("уводит на чёрный экран по событию из Rust", async () => {
    renderApp();
    await waitFor(() => expect(listeners.size).toBe(2));

    emit("lokked://zen");

    expect(await screen.findByText("Чёрный экран")).toBeInTheDocument();
  });

  it("открывает чёрный экран, если о нём просили ещё до запуска окна", async () => {
    invoke.mockImplementation((command: string) =>
      command === "cli_pending_zen"
        ? Promise.resolve(true)
        : Promise.reject(new Error(command)),
    );

    renderApp();

    expect(await screen.findByText("Чёрный экран")).toBeInTheDocument();
  });

  it("без такой просьбы остаётся там, где был", async () => {
    renderApp();

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("cli_pending_zen");
    });
    expect(screen.getByText("Таймеры")).toBeInTheDocument();
  });
});

describe("пробуждение машины", () => {
  it("предлагает продолжить и говорит, сколько машина спала", async () => {
    renderApp();
    await waitFor(() => expect(listeners.size).toBe(2));

    emit("lokked://woke", { asleep_seconds: 3600 });

    expect(await screen.findByText("С возвращением")).toBeInTheDocument();
    expect(screen.getByText(/Компьютер спал 1 ч/)).toBeInTheDocument();
  });

  it("«Продолжить» снимает паузу", async () => {
    renderApp();
    await waitFor(() => expect(listeners.size).toBe(2));
    emit("lokked://woke", { asleep_seconds: 3600 });
    await screen.findByText("С возвращением");

    await userEvent.click(screen.getByRole("button", { name: "Продолжить" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("resume_session");
    });
    expect(screen.queryByText("С возвращением")).not.toBeInTheDocument();
  });

  it("«Оставить на паузе» ничего не трогает", async () => {
    renderApp();
    await waitFor(() => expect(listeners.size).toBe(2));
    emit("lokked://woke", { asleep_seconds: 60 });
    await screen.findByText("С возвращением");

    await userEvent.click(
      screen.getByRole("button", { name: "Оставить на паузе" }),
    );

    await waitFor(() => {
      expect(screen.queryByText("С возвращением")).not.toBeInTheDocument();
    });
    expect(invoke).not.toHaveBeenCalledWith("resume_session");
  });

  it("отписывается, когда уходит с экрана", async () => {
    const { unmount } = renderApp();
    await waitFor(() => expect(listeners.size).toBe(2));

    unmount();

    await waitFor(() => expect(unlisten).toHaveBeenCalled());
  });
});
