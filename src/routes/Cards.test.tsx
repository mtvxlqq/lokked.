import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Cards } from "@/routes/Cards";
import type { Card, Deck, ImportReport, Subject } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const subject: Subject = {
  id: "s-1",
  name: "Математический анализ",
  color: "subject-1",
  icon: null,
  position: 0,
};

const deck: Deck = {
  id: "d-1",
  subject_id: "s-1",
  name: "Терсенов, § 25 — § 40",
  description: "лекции",
  card_count: 2,
};

const other: Deck = {
  id: "d-2",
  subject_id: null,
  name: "Разное",
  description: null,
  card_count: 0,
};

const cards: Card[] = [
  {
    id: "c-1",
    deck_id: "d-1",
    front: "Первообразная функции",
    back: "Функция $F(x)$, для которой $F'(x)=f(x)$",
    hint: null,
    tags: ["определение", "лекция 25"],
  },
  {
    id: "c-2",
    deck_id: "d-1",
    front: "Признак Даламбера",
    back: "Если предел отношения меньше единицы, ряд сходится",
    hint: null,
    tags: ["признак", "лекция 30"],
  },
];

/** Отвечает на команды экрана; данные задаются тем, что лежит в базе. */
function backend(
  data: {
    decks?: Deck[];
    cards?: Card[];
    report?: Partial<ImportReport>;
    imported?: number;
  } = {},
) {
  invoke.mockImplementation((command: string, args?: unknown) => {
    switch (command) {
      case "list_decks":
        return Promise.resolve(data.decks ?? [deck, other]);
      case "list_subjects":
        return Promise.resolve([subject]);
      case "list_cards": {
        const { deckId } = (args ?? {}) as { deckId: string };
        return Promise.resolve(
          (data.cards ?? cards).filter((card) => card.deck_id === deckId),
        );
      }
      case "preview_import":
        return Promise.resolve({
          format: "text",
          cards: [
            { front: "Лицо", back: "Оборот", hint: null, tags: [] },
            { front: "Второе", back: "Оборот", hint: null, tags: [] },
          ],
          problems: [],
          suggested_deck: null,
          suggested_description: null,
          ...data.report,
        } satisfies ImportReport);
      case "import_cards":
        return Promise.resolve(data.imported ?? 2);
      case "create_deck":
        return Promise.resolve({ ...other, id: "d-3", name: "Новая" });
      case "export_deck":
        return Promise.resolve("Лицо\n---\nОборот");
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

/** Экран живёт в роутере: с него уходят на прогон по колоде. */
function renderCards() {
  const router = createMemoryRouter(
    [
      { path: "/cards", element: <Cards /> },
      { path: "/study/:deckId", element: <p>прогон по колоде</p> },
    ],
    { initialEntries: ["/cards"] },
  );

  return { ...render(<RouterProvider router={router} />), router };
}

/** Ждёт, пока экран загрузился: список карточек при открытии свёрнут. */
async function ready() {
  return screen.findByRole("button", { name: "Показать карточки" });
}

/** Разворачивает список карточек. */
async function openCards() {
  await userEvent.click(await ready());
}

beforeEach(() => {
  invoke.mockReset();
});

describe("экран карточек", () => {
  it("предлагает импорт, когда колод ещё нет", async () => {
    backend({ decks: [] });
    renderCards();

    expect(await screen.findByText("Колод пока нет")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Импортировать" }),
    ).toBeInTheDocument();
  });

  it("открывает первую колоду сам, но список карточек держит свёрнутым", async () => {
    backend();
    renderCards();

    // Колода выбрана, карточек на экране пока нет: сюда заходят учить.
    expect(
      await screen.findByText("2 карточки · Математический анализ"),
    ).toBeInTheDocument();
    expect(screen.queryByText("Первообразная функции")).not.toBeInTheDocument();

    await openCards();

    expect(
      await screen.findByText("Первообразная функции"),
    ).toBeInTheDocument();
    expect(screen.getByText("Признак Даламбера")).toBeInTheDocument();
  });

  it("в списке показывает формулы, а не их исходник", async () => {
    backend();
    const { container } = renderCards();
    await openCards();
    await screen.findByText("Первообразная функции");

    // У второй карточки формул нет, у первой — две в обороте.
    expect(container.querySelectorAll(".katex").length).toBeGreaterThan(0);
    expect(screen.queryByText(/\$F'\(x\)=f\(x\)\$/)).not.toBeInTheDocument();
  });

  it("жирный, внутри которого формула, показывается жирным", async () => {
    backend({
      cards: [
        {
          id: "c-3",
          deck_id: "d-1",
          front: "Непрерывное отображение",
          back: "Отображение называется **непрерывным в точке $x_0 \\in X$**, если",
          hint: null,
          tags: [],
        },
      ],
    });
    const { container } = renderCards();
    await openCards();
    await screen.findByText("Непрерывное отображение");

    expect(container.querySelector("strong")).not.toBeNull();
    expect(screen.queryByText(/\*\*/)).not.toBeInTheDocument();
  });

  it("формулу в теге рисует формулой", async () => {
    backend({
      cards: [
        {
          id: "c-4",
          deck_id: "d-1",
          front: "Теорема Кантора",
          back: "Оборот",
          hint: null,
          tags: ["Кривые и области в $\\mathbb{R}^m$"],
        },
      ],
    });
    const { container } = renderCards();
    await openCards();
    await screen.findByText("Теорема Кантора");

    // KaTeX оставляет исходник формулы в MathML для копирования и
    // скринридеров, поэтому проверяем, что на экране нет долларов, —
    // видимого исходника.
    expect(container.querySelectorAll(".katex").length).toBeGreaterThan(0);
    expect(screen.queryByText(/\$/)).not.toBeInTheDocument();
  });

  it("переключает колоду по нажатию", async () => {
    backend();
    renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: /^Разное/ }));
    await openCards();

    // Вторая колода пуста, и это видно, когда список развёрнут.
    expect(
      await screen.findByText("В колоде пока нет карточек."),
    ).toBeInTheDocument();
  });

  it("ищет по тексту карточки", async () => {
    backend();
    renderCards();
    await openCards();
    await screen.findByText("Первообразная функции");

    await userEvent.type(screen.getByLabelText("Поиск"), "Даламбер");

    expect(screen.queryByText("Первообразная функции")).not.toBeInTheDocument();
    expect(screen.getByText("Признак Даламбера")).toBeInTheDocument();
    expect(screen.getByText("Показано 1 из 2")).toBeInTheDocument();
  });

  it("фильтрует по тегу и снимает фильтр повторным нажатием", async () => {
    backend();
    renderCards();
    await openCards();
    await screen.findByText("Первообразная функции");

    const tag = screen.getByRole("button", { name: /^определение/ });
    await userEvent.click(tag);
    expect(screen.queryByText("Признак Даламбера")).not.toBeInTheDocument();

    await userEvent.click(tag);
    expect(screen.getByText("Признак Даламбера")).toBeInTheDocument();
  });

  it("импортирует разобранные карточки в выбранную колоду", async () => {
    backend();
    renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: "Импорт" }));
    await userEvent.type(
      screen.getByLabelText("Или вставь текст"),
      "Лицо{Enter}---{Enter}Оборот",
    );
    await userEvent.click(screen.getByRole("button", { name: "Разобрать" }));

    expect(
      await screen.findByText("Распознано карточек: 2"),
    ).toBeInTheDocument();

    await userEvent.click(
      screen.getByRole("button", { name: "Импортировать" }),
    );

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("import_cards", {
        deckId: "d-1",
        cards: [
          { front: "Лицо", back: "Оборот", hint: null, tags: [] },
          { front: "Второе", back: "Оборот", hint: null, tags: [] },
        ],
      }),
    );
  });

  it("показывает, какие блоки импорта не разобрались", async () => {
    backend({
      report: {
        cards: [{ front: "Лицо", back: "Оборот", hint: null, tags: [] }],
        problems: [{ block: 2, kind: "missing_back" }],
      },
    });
    renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: "Импорт" }));
    await userEvent.click(screen.getByRole("button", { name: "Разобрать" }));

    expect(
      await screen.findByText("блок 2: нет оборотной стороны"),
    ).toBeInTheDocument();
  });

  it("отдаёт колоду текстом на экспорт", async () => {
    backend();
    renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: "Экспорт" }));

    expect(await screen.findByLabelText("Карточки")).toHaveValue(
      "Лицо\n---\nОборот",
    );
  });

  it("список карточек разворачивается и сворачивается обратно", async () => {
    backend();
    renderCards();
    await openCards();
    await screen.findByText("Первообразная функции");

    await userEvent.click(
      screen.getByRole("button", { name: "Скрыть карточки" }),
    );

    expect(screen.queryByText("Первообразная функции")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Поиск")).not.toBeInTheDocument();
    // Колода на месте — и в списке слева, и заголовком справа.
    expect(screen.getAllByText(/Терсенов/).length).toBeGreaterThan(0);

    await userEvent.click(
      screen.getByRole("button", { name: "Показать карточки" }),
    );
    expect(
      await screen.findByText("Первообразная функции"),
    ).toBeInTheDocument();
  });

  it("каждый режим уводит на свой прогон по этой колоде", async () => {
    backend();
    const { router } = renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: "Классика" }));
    expect(await screen.findByText("прогон по колоде")).toBeInTheDocument();
    expect(router.state.location.search).toBe("?mode=classic");
  });

  it("блиц, марафон и слабые — отдельные кнопки", async () => {
    backend();
    const { router } = renderCards();
    await ready();

    await userEvent.click(screen.getByRole("button", { name: "Блиц" }));

    expect(await screen.findByText("прогон по колоде")).toBeInTheDocument();
    expect(router.state.location.search).toBe("?mode=blitz");
    expect(router.state.location.pathname).toBe("/study/d-1");
  });

  it("сообщает об отказе команды и даёт повторить", async () => {
    invoke.mockRejectedValue({
      kind: "database",
      message: "database query failed: disk I/O error",
    });
    renderCards();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "disk I/O error",
    );

    backend();
    await userEvent.click(screen.getByRole("button", { name: "Повторить" }));

    expect(await ready()).toBeInTheDocument();
  });
});
