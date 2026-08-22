import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Duel } from "@/routes/Duel";
import type { Deck, DuelSummary, DuelView, Grade } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const setFullscreen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setFullscreen }),
}));

const decks: Deck[] = [
  {
    id: "d-1",
    subject_id: null,
    name: "Линейная алгебра",
    description: null,
    card_count: 126,
  },
  {
    id: "d-2",
    subject_id: null,
    name: "Матанализ",
    description: null,
    card_count: 40,
  },
];

const cards = [
  { id: "c-1", front: "Ортогональное дополнение", back: "$V^\\perp$" },
  { id: "c-2", front: "Критерий обратимости", back: "det ≠ 0" },
];

/**
 * Бэкенд дуэли: держит очередь, ходы и ответы — как настоящий, только в
 * памяти теста. Последовательность у всех игроков одна и та же.
 */
function backend(options: { players?: string[]; cards?: number } = {}) {
  const players = options.players ?? ["Ты", "Артём"];
  const total = options.cards ?? 2;
  let current = 0;
  let position = 0;
  let revealed = false;
  let handover = true;
  let finished = false;
  const answers: Grade[][] = players.map(() => []);

  const points = (player: number) =>
    answers[player].filter((grade) => grade !== "again").length * 10;

  const view = (): DuelView => ({
    duel_id: "duel-1",
    deck_id: "d-1",
    deck_name: "Линейная алгебра",
    players: players.map((name, index) => ({
      name,
      is_owner: index === 0,
      played: finished || index < current,
    })),
    current_player: current,
    current_name: players[current],
    turn: current + 1,
    turns: players.length,
    total,
    position: Math.min(position + 1, total),
    answered: answers[current].length,
    revealed,
    card:
      handover || finished
        ? null
        : {
            ...cards[position % cards.length],
            back: revealed ? cards[position % cards.length].back : null,
            hint: null,
            tags: [],
          },
    deadline:
      handover || finished ? null : new Date(Date.now() + 20_000).toISOString(),
    seconds_per_card: 20,
    points: points(current),
    streak: 0,
    handover,
    finished,
  });

  const summary = (): DuelSummary => ({
    duel_id: "duel-1",
    deck_id: "d-1",
    deck_name: "Линейная алгебра",
    cards: total,
    seconds_per_card: 20,
    players: players.map((name, index) => ({
      name,
      is_owner: index === 0,
      points: points(index),
      correct: answers[index].filter((grade) => grade !== "again").length,
      answered: answers[index].length,
      best_streak: 1,
      winner: points(index) === Math.max(...players.map((_, i) => points(i))),
    })),
    breakdown: cards.slice(0, total).map((card, index) => ({
      card_id: card.id,
      front: card.front,
      back: card.back,
      answers: players.map((_, player) => answers[player][index] ?? null),
    })),
  });

  invoke.mockImplementation((command: string, args?: unknown) => {
    switch (command) {
      case "list_decks":
        return Promise.resolve(decks);
      case "list_cards":
        return Promise.resolve(
          cards.map((card) => ({
            ...card,
            deck_id: "d-1",
            hint: null,
            tags: [],
          })),
        );
      case "duel_current":
        return Promise.resolve(null);
      case "duel_pick_deck":
        return Promise.resolve(decks[1]);
      case "duel_start":
        return Promise.resolve(view());
      case "duel_begin_turn":
        handover = false;
        position = 0;
        revealed = false;
        return Promise.resolve(view());
      case "duel_settled":
        return Promise.resolve(view());
      case "duel_reveal":
        revealed = true;
        return Promise.resolve(view());
      case "duel_answer": {
        const { grade } = args as { grade: Grade };
        answers[current].push(grade);
        position += 1;
        revealed = false;

        if (position >= total) {
          if (current + 1 < players.length) {
            current += 1;
            position = 0;
            handover = true;
          } else {
            finished = true;
          }
        }
        return Promise.resolve(view());
      }
      case "duel_summary":
        return Promise.resolve(summary());
      case "duel_stop":
        return Promise.resolve(null);
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

function renderDuel() {
  const router = createMemoryRouter(
    [
      { path: "/duel", element: <Duel /> },
      { path: "/cards", element: <p>экран карточек</p> },
    ],
    { initialEntries: ["/duel"] },
  );

  render(<RouterProvider router={router} />);
  return router;
}

/** Раскрывает карточку и отвечает «Знаю». */
async function answerCard() {
  await userEvent.click(
    await screen.findByRole("button", { name: "Показать ответ" }),
  );
  await userEvent.click(await screen.findByRole("button", { name: /Знаю/ }));
}

/** Дожидается барабана и начинает ход текущего игрока. */
async function takeTurn() {
  await userEvent.click(
    await screen.findByRole("button", { name: "Я готов — начать" }),
  );
}

beforeEach(() => {
  invoke.mockReset();
  setFullscreen.mockReset();
  setFullscreen.mockResolvedValue(undefined);
});

describe("дуэль", () => {
  it("начинается с настройки: игроки, колода, условия", async () => {
    backend();
    renderDuel();

    expect(await screen.findByLabelText("Хозяин устройства")).toHaveValue("Ты");
    expect(screen.getByLabelText("Игрок 2")).toHaveValue("Артём");
    expect(screen.getByLabelText("На чём соревнуемся")).toBeInTheDocument();
    expect(screen.getByLabelText("Карточек на игрока")).toHaveValue("20");
  });

  it("барабан выбирает колоду за тебя", async () => {
    backend();
    renderDuel();

    await userEvent.click(
      await screen.findByRole("button", { name: "Крутить барабан" }),
    );

    // Пока крутится — только барабан; когда встал, можно идти дальше.
    expect(
      await screen.findByText("Барабан выбирает колоду"),
    ).toBeInTheDocument();
    await userEvent.click(
      await screen.findByRole("button", { name: "Дальше" }),
    );

    expect(
      await screen.findByText("Барабан выбрал: Матанализ"),
    ).toBeInTheDocument();
  });

  it("не даёт начать дуэль на колоде, в которой мало карточек", async () => {
    backend();
    renderDuel();
    await screen.findByLabelText("На чём соревнуемся");

    await userEvent.selectOptions(
      screen.getByLabelText("На чём соревнуемся"),
      "d-2",
    );
    await userEvent.selectOptions(
      screen.getByLabelText("Карточек на игрока"),
      "50",
    );

    expect(
      screen.getByText(/В колоде 40 карточек — на дуэль из 50 не хватит/),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Начать дуэль" })).toBeDisabled();
  });

  it("после старта ждёт, пока первый игрок возьмёт устройство", async () => {
    backend();
    renderDuel();
    await screen.findByLabelText("На чём соревнуемся");

    await userEvent.selectOptions(
      screen.getByLabelText("На чём соревнуемся"),
      "d-1",
    );
    await userEvent.click(screen.getByRole("button", { name: "Начать дуэль" }));

    expect(await screen.findByText("Дуэль начинается")).toBeInTheDocument();
    expect(screen.getByText("Ты")).toBeInTheDocument();
    expect(screen.getByText("1 / 2")).toBeInTheDocument();
    // Карточки на экране нет, пока не сказали «готов».
    expect(
      screen.queryByText("Ортогональное дополнение"),
    ).not.toBeInTheDocument();
  });

  it("передаёт устройство между ходами, пряча чужой счёт", async () => {
    backend({ cards: 1 });
    renderDuel();
    await startDuel();

    await takeTurn();
    await answerCard();

    expect(await screen.findByText("Передай устройство")).toBeInTheDocument();
    expect(screen.getByText("Артём")).toBeInTheDocument();
    expect(
      screen.getByText("Результат предыдущего игрока скрыт до конца дуэли."),
    ).toBeInTheDocument();
    // Счёта первого игрока на экране нет.
    expect(screen.queryByText("10")).not.toBeInTheDocument();
  });

  it("показывает итоги с победителем и разбором по карточкам", async () => {
    backend({ cards: 1 });
    renderDuel();
    await startDuel();

    await takeTurn();
    await answerCard();
    await takeTurn();
    await userEvent.click(
      await screen.findByRole("button", { name: "Показать ответ" }),
    );
    await userEvent.click(
      await screen.findByRole("button", { name: /Не помню/ }),
    );

    const winner = within(await screen.findByRole("group", { name: "Ты" }));
    expect(winner.getByText("10")).toBeInTheDocument();
    expect(winner.getByText(/победа/)).toBeInTheDocument();

    const table = screen.getByRole("table");
    expect(
      within(table).getByText("Ортогональное дополнение"),
    ).toBeInTheDocument();
    expect(within(table).getByText("Знаю")).toBeInTheDocument();
    expect(within(table).getByText("Не помню")).toBeInTheDocument();
  });

  it("«ещё раз» начинает новую дуэль с теми же игроками", async () => {
    backend({ cards: 1 });
    renderDuel();
    await startDuel();
    await takeTurn();
    await answerCard();
    await takeTurn();
    await answerCard();

    await userEvent.click(
      await screen.findByRole("button", { name: "Ещё раз" }),
    );

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("duel_start", {
        deckId: "d-1",
        players: ["Ты", "Артём"],
        cards: 1,
        secondsPerCard: 20,
      }),
    );
  });

  it("выход прерывает дуэль и возвращает к карточкам", async () => {
    backend({ cards: 1 });
    renderDuel();
    await startDuel();

    await userEvent.click(
      await screen.findByRole("button", { name: "Прервать дуэль" }),
    );

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("duel_stop"));
    expect(await screen.findByText("экран карточек")).toBeInTheDocument();
  });
});

/** Выбирает колоду и начинает дуэль. */
async function startDuel() {
  await screen.findByLabelText("На чём соревнуемся");
  await userEvent.selectOptions(
    screen.getByLabelText("На чём соревнуемся"),
    "d-1",
  );
  await userEvent.click(screen.getByRole("button", { name: "Начать дуэль" }));
}
