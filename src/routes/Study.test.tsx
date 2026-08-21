import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Study } from "@/routes/Study";
import type { Grade, StudyMode, StudySummary, StudyView } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const cards = [
  { id: "c-1", front: "Первообразная", back: "$F'=f$", hint: null, tags: [] },
  { id: "c-2", front: "Интеграл", back: "множество", hint: null, tags: [] },
];

/**
 * Бэкенд прогона: держит очередь, раскрытие и ответы — ровно как настоящий,
 * только в памяти теста.
 */
function backend(options: { total?: number; mode?: StudyMode } = {}) {
  const total = options.total ?? 2;
  const mode = options.mode ?? "classic";
  const timed = mode === "blitz";
  let position = 0;
  let revealed = false;
  const grades: Grade[] = [];

  const points = () => grades.filter((grade) => grade !== "again").length * 10;

  const view = (): StudyView => ({
    deck_id: "d-1",
    deck_name: "Матанализ",
    mode,
    total,
    position: Math.min(position + 1, total),
    answered: grades.length,
    revealed,
    card:
      position < total
        ? { ...cards[position], back: revealed ? cards[position].back : null }
        : null,
    finished: position >= total,
    // Дедлайн у блица всегда через 20 секунд от «сейчас» теста.
    deadline:
      timed && position < total
        ? new Date(Date.now() + 20_000).toISOString()
        : null,
    seconds_per_card: timed ? 20 : null,
    points: timed ? points() : null,
    streak: timed ? 0 : null,
  });

  const summary = (): StudySummary => {
    const mistakes = grades
      .map((grade, index) => ({ grade, index }))
      .filter((answer) => answer.grade === "again");

    return {
      deck_id: "d-1",
      deck_name: "Матанализ",
      mode,
      answered: grades.length,
      correct: grades.length - mistakes.length,
      accuracy_percent: Math.round(
        ((grades.length - mistakes.length) / Math.max(1, grades.length)) * 100,
      ),
      total_ms: 12_000,
      average_ms: 6_000,
      mistakes: mistakes.map((answer) => cards[answer.index].id),
      mistake_cards: mistakes.map((answer) => cards[answer.index]),
      points: timed ? points() : null,
      best_streak: timed ? 1 : null,
      record: timed ? 40 : null,
      record_beaten: false,
    };
  };

  invoke.mockImplementation((command: string, args?: unknown) => {
    switch (command) {
      case "study_current":
        return Promise.resolve(
          grades.length === 0 && position === 0 ? null : view(),
        );
      case "study_start":
        position = 0;
        revealed = false;
        grades.length = 0;
        return Promise.resolve(view());
      case "study_reveal":
        revealed = true;
        return Promise.resolve(view());
      case "study_answer": {
        const { grade } = args as { grade: Grade };
        grades.push(grade);
        position += 1;
        revealed = false;
        return Promise.resolve(view());
      }
      case "study_timeout":
        grades.push("again");
        position += 1;
        revealed = false;
        return Promise.resolve(view());
      case "study_summary":
        return Promise.resolve(summary());
      case "study_repeat_mistakes":
        position = 0;
        revealed = false;
        grades.length = 0;
        return Promise.resolve({ ...view(), total: 1 });
      case "study_stop":
        return Promise.resolve(null);
      case "list_cards":
        // Барабану нужны лицевые стороны колоды для ленты прокрутки.
        return Promise.resolve(
          cards.map((card) => ({ ...card, deck_id: "d-1", back: card.back })),
        );
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

function renderStudy(mode?: StudyMode) {
  const router = createMemoryRouter(
    [
      { path: "/study/:deckId", element: <Study /> },
      { path: "/cards", element: <p>экран карточек</p> },
    ],
    { initialEntries: [mode ? `/study/d-1?mode=${mode}` : "/study/d-1"] },
  );

  render(<RouterProvider router={router} />);
  return router;
}

beforeEach(() => {
  invoke.mockReset();
});

describe("прогон по колоде", () => {
  it("показывает первую карточку без ответа", async () => {
    backend();
    renderStudy();

    expect(await screen.findByText("Первообразная")).toBeInTheDocument();
    expect(screen.getByText("1 / 2")).toBeInTheDocument();
    expect(
      screen.getByText("Проговори формулировку вслух, затем пробел"),
    ).toBeInTheDocument();
    // Оценок до раскрытия нет.
    expect(
      screen.queryByRole("button", { name: /Знаю/ }),
    ).not.toBeInTheDocument();
  });

  it("по пробелу раскрывает ответ", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");

    expect(
      await screen.findByRole("button", { name: /Знаю/ }),
    ).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("study_reveal");
  });

  it("по нажатию на карточку тоже раскрывает", async () => {
    backend();
    renderStudy();

    // Нажатие по самой карточке, а не по кнопке под ней.
    await userEvent.click(
      await screen.findByRole("button", {
        name: "Карточка, нажми, чтобы увидеть ответ",
      }),
    );

    expect(
      await screen.findByRole("button", { name: /Знаю/ }),
    ).toBeInTheDocument();
  });

  it("оценка с клавиатуры отправляется и открывает следующую карточку", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await screen.findByRole("button", { name: /Знаю/ });
    await userEvent.keyboard("3");

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("study_answer", { grade: "good" }),
    );
    expect(await screen.findByText("Интеграл")).toBeInTheDocument();
    expect(screen.getByText("2 / 2")).toBeInTheDocument();
  });

  it("оценка мышью работает так же", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await userEvent.click(
      await screen.findByRole("button", { name: /Не помню/ }),
    );

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("study_answer", { grade: "again" }),
    );
  });

  it("после последней карточки показывает итоги с разбором ошибок", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await userEvent.keyboard("1");
    await screen.findByText("Интеграл");
    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");

    expect(await screen.findByText("Прогон закончен")).toBeInTheDocument();
    expect(screen.getByText("50%")).toBeInTheDocument();
    // Ошибочная карточка показана вместе с ответом.
    expect(screen.getByText("Ошибки")).toBeInTheDocument();
    expect(screen.getAllByText("Первообразная").length).toBeGreaterThan(0);
  });

  it("«Повторить ошибки» начинает новый прогон", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await userEvent.keyboard("1");
    await screen.findByText("Интеграл");
    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");
    await screen.findByText("Прогон закончен");

    await userEvent.click(
      screen.getByRole("button", { name: "Повторить ошибки" }),
    );

    expect(invoke).toHaveBeenCalledWith("study_repeat_mistakes");
    expect(await screen.findByText("1 / 1")).toBeInTheDocument();
  });

  it("Esc заканчивает прогон и возвращает к карточкам", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard("{Escape}");

    expect(await screen.findByText("экран карточек")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("study_stop");
  });

  it("сообщает, если колоду учить нельзя", async () => {
    invoke.mockImplementation((command: string) =>
      command === "study_current"
        ? Promise.resolve(null)
        : Promise.reject({
            kind: "conflict",
            message: "в колоде нет карточек",
          }),
    );
    renderStudy();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "в колоде нет карточек",
    );
  });

  it("режим берётся из адреса и уходит в бэкенд", async () => {
    backend({ mode: "marathon" });
    renderStudy("marathon");
    await screen.findByText("Первообразная");

    expect(invoke).toHaveBeenCalledWith("study_start", {
      deckId: "d-1",
      mode: "marathon",
    });
    expect(screen.getByText(/Марафон/)).toBeInTheDocument();
    // У марафона есть полоса прогресса.
    expect(screen.getByRole("progressbar")).toBeInTheDocument();
  });

  it("блиц показывает обратный отсчёт и очки", async () => {
    backend({ mode: "blitz" });
    renderStudy("blitz");
    await screen.findByText("Первообразная");

    expect(screen.getByRole("timer")).toBeInTheDocument();
    expect(screen.getByText("0")).toBeInTheDocument();
    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
  });

  it("когда время карточки вышло, ответ отправляется сам", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      backend({ mode: "blitz" });
      renderStudy("blitz");
      await screen.findByText("Первообразная");

      await act(async () => {
        vi.advanceTimersByTime(21_000);
      });

      expect(invoke).toHaveBeenCalledWith("study_timeout");
      expect(await screen.findByText("Интеграл")).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("итоги блица показывают счёт и рекорд колоды", async () => {
    backend({ mode: "blitz" });
    renderStudy("blitz");
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");
    await screen.findByText("Интеграл");
    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");

    expect(await screen.findByText("Счёт")).toBeInTheDocument();
    expect(screen.getByText("20")).toBeInTheDocument();
    expect(screen.getByText("рекорд колоды — 40")).toBeInTheDocument();
  });

  it("у обычного прогона счёта нет", async () => {
    backend();
    renderStudy();
    await screen.findByText("Первообразная");

    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");
    await screen.findByText("Интеграл");
    await userEvent.keyboard(" ");
    await userEvent.keyboard("3");

    await screen.findByText("Прогон закончен");
    expect(screen.queryByText("Счёт")).not.toBeInTheDocument();
  });
});

describe("барабан", () => {
  /**
   * Прокручивает барабан до остановки.
   *
   * Два шага, потому что их два и в жизни: сначала кадр, на котором лента
   * трогается с места, и только потом — время самой прокрутки.
   */
  async function spin() {
    // Крутить нечего, пока прогон не начался: сперва дожидаемся барабана.
    await screen.findByRole("status");
    await act(async () => {
      vi.advanceTimersByTime(50);
    });
    await act(async () => {
      vi.advanceTimersByTime(1500);
    });
  }

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("сначала крутится, потом показывает выпавшую карточку", async () => {
    backend({ mode: "reel" });
    renderStudy("reel");

    expect(
      await screen.findByRole("status", { name: "Барабан крутится" }),
    ).toBeInTheDocument();

    await spin();

    expect(
      screen.getByRole("status", { name: "Выпало: Первообразная" }),
    ).toBeInTheDocument();
  });

  it("пока барабан крутится, ответ не раскрыть", async () => {
    backend({ mode: "reel" });
    renderStudy("reel");
    await screen.findByRole("status", { name: "Барабан крутится" });

    await userEvent.keyboard(" ");

    expect(invoke).not.toHaveBeenCalledWith("study_reveal");
    // И кнопки, по которой можно было бы нажать, тоже нет.
    expect(
      screen.queryByRole("button", { name: "Показать ответ" }),
    ).not.toBeInTheDocument();
  });

  it("когда барабан встал, пробел показывает ответ", async () => {
    backend({ mode: "reel" });
    renderStudy("reel");
    await spin();

    await userEvent.keyboard(" ");

    expect(invoke).toHaveBeenCalledWith("study_reveal");
    // Оборот первой карточки — формула, её рисует KaTeX; проверяем по тому,
    // что рядом с ответом появились оценки.
    expect(
      await screen.findByText("Насколько уверенно ты ответил?"),
    ).toBeInTheDocument();
  });

  it("оценка крутит барабан заново для следующей карточки", async () => {
    backend({ mode: "reel" });
    renderStudy("reel");
    await spin();
    await userEvent.keyboard(" ");
    await screen.findByText("Насколько уверенно ты ответил?");

    await userEvent.keyboard("3");

    expect(
      await screen.findByRole("status", { name: "Барабан крутится" }),
    ).toBeInTheDocument();
    await spin();
    expect(
      screen.getByRole("status", { name: "Выпало: Интеграл" }),
    ).toBeInTheDocument();
  });

  it("в ленте нет оборотов карточек", async () => {
    // Иначе барабан подсказывал бы ответ ещё до того, как остановится.
    backend({ mode: "reel" });
    renderStudy("reel");
    await spin();

    expect(screen.queryByText("множество")).not.toBeInTheDocument();
  });

  it("выход по Esc возвращает на экран карточек", async () => {
    backend({ mode: "reel" });
    renderStudy("reel");
    await spin();

    await userEvent.keyboard("{Escape}");

    expect(await screen.findByText("экран карточек")).toBeInTheDocument();
  });
});
