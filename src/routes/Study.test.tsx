import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Study } from "@/routes/Study";
import type { Grade, StudySummary, StudyView } from "@/lib/tauri";

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
function backend(options: { total?: number } = {}) {
  const total = options.total ?? 2;
  let position = 0;
  let revealed = false;
  const grades: Grade[] = [];

  const view = (): StudyView => ({
    deck_id: "d-1",
    deck_name: "Матанализ",
    mode: "classic",
    total,
    position: Math.min(position + 1, total),
    answered: grades.length,
    revealed,
    card:
      position < total
        ? { ...cards[position], back: revealed ? cards[position].back : null }
        : null,
    finished: position >= total,
  });

  const summary = (): StudySummary => {
    const mistakes = grades
      .map((grade, index) => ({ grade, index }))
      .filter((answer) => answer.grade === "again");

    return {
      deck_id: "d-1",
      deck_name: "Матанализ",
      answered: grades.length,
      correct: grades.length - mistakes.length,
      accuracy_percent: Math.round(
        ((grades.length - mistakes.length) / Math.max(1, grades.length)) * 100,
      ),
      total_ms: 12_000,
      average_ms: 6_000,
      mistakes: mistakes.map((answer) => cards[answer.index].id),
      mistake_cards: mistakes.map((answer) => cards[answer.index]),
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
      case "study_summary":
        return Promise.resolve(summary());
      case "study_repeat_mistakes":
        position = 0;
        revealed = false;
        grades.length = 0;
        return Promise.resolve({ ...view(), total: 1 });
      case "study_stop":
        return Promise.resolve(null);
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

function renderStudy() {
  const router = createMemoryRouter(
    [
      { path: "/study/:deckId", element: <Study /> },
      { path: "/cards", element: <p>экран карточек</p> },
    ],
    { initialEntries: ["/study/d-1"] },
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
});
