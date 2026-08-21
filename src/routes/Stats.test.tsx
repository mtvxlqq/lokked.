import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Stats } from "@/routes/Stats";
import type {
  Card,
  CardReport,
  CardsStats,
  Deck,
  HeatCell,
  Subject,
  TimeStats,
} from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const subjects: Subject[] = [
  {
    id: "s-1",
    name: "Математический анализ",
    color: "subject-1",
    icon: null,
    position: 0,
  },
  {
    id: "s-2",
    name: "Физика",
    color: "subject-5",
    icon: null,
    position: 1,
  },
];

const deck: Deck = {
  id: "d-1",
  subject_id: "s-1",
  name: "Терсенов, § 25 — § 40",
  description: null,
  card_count: 2,
};

const cards: Card[] = [
  {
    id: "c-1",
    deck_id: "d-1",
    front: "Критерий обратимости матрицы",
    back: "Определитель отличен от нуля",
    hint: null,
    tags: [],
  },
  {
    id: "c-2",
    deck_id: "d-1",
    front: "Признак Даламбера",
    back: "Предел отношения меньше единицы",
    hint: null,
    tags: [],
  },
];

/** Неделя клеток, чтобы карта активности была не пустой. */
const heatmap: HeatCell[] = Array.from({ length: 7 }, (_, index) => ({
  day_key: `2026-08-${15 + index}`,
  seconds: index * 600,
  level: Math.min(index, 4),
  weekday: (5 + index) % 7,
}));

const timeStats: TimeStats = {
  range: "week",
  from: "2026-08-15",
  to: "2026-08-21",
  total_seconds: 9000,
  pomodoros: 3,
  streak_days: 12,
  subjects: [
    { subject_id: "s-1", seconds: 6000, share_percent: 100 },
    { subject_id: "s-2", seconds: 3000, share_percent: 50 },
  ],
  heatmap,
};

const cardsStats: CardsStats = {
  range: "week",
  from: "2026-08-15",
  to: "2026-08-21",
  answered: 40,
  correct: 30,
  accuracy_percent: 75,
  by_day: [
    {
      day_key: "2026-08-20",
      answered: 0,
      correct: 0,
      accuracy_percent: 0,
    },
    {
      day_key: "2026-08-21",
      answered: 40,
      correct: 30,
      accuracy_percent: 75,
    },
  ],
  problems: [
    {
      card_id: "c-1",
      shown: 10,
      correct: 2,
      accuracy_percent: 20,
      front: "Критерий обратимости матрицы",
      deck_id: "d-1",
    },
  ],
};

const report: CardReport = {
  card_id: "c-1",
  deck_id: "d-1",
  front: "Критерий обратимости матрицы",
  back: "Определитель отличен от нуля",
  shown: 10,
  correct: 2,
  accuracy_percent: 20,
  recent: ["again", "again", "good"],
  average_think_ms: 4200,
  current_streak: 1,
};

/** Отвечает на команды экрана; чем именно — задаётся аргументом. */
function backend(
  data: {
    time?: TimeStats;
    cards?: CardsStats;
    report?: CardReport;
    csv?: string;
    failing?: string;
  } = {},
) {
  invoke.mockImplementation((command: string) => {
    if (command === data.failing) {
      return Promise.reject({ kind: "database", message: "база недоступна" });
    }

    switch (command) {
      case "stats_time":
        return Promise.resolve(data.time ?? timeStats);
      case "stats_cards":
        return Promise.resolve(data.cards ?? cardsStats);
      case "stats_card":
        return Promise.resolve(data.report ?? report);
      case "stats_export_csv":
        return Promise.resolve(data.csv ?? "день,секунды\n2026-08-21,9000\n");
      case "list_subjects":
        return Promise.resolve(subjects);
      case "list_decks":
        return Promise.resolve([deck]);
      case "list_cards":
        return Promise.resolve(cards);
      default:
        return Promise.reject(new Error(`неожиданная команда: ${command}`));
    }
  });
}

/** Аргументы, с которыми вызывали команду. */
function callsOf(command: string) {
  return invoke.mock.calls.filter(([name]) => name === command);
}

beforeEach(() => {
  invoke.mockReset();
});

describe("вкладка «Время»", () => {
  it("показывает время, помодоро и серию за период", async () => {
    backend();
    render(<Stats />);

    expect(await screen.findByText("2 ч 30 мин")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("12 дней")).toBeInTheDocument();
  });

  it("разбивает время по предметам", async () => {
    backend();
    render(<Stats />);

    expect(
      await screen.findByText("Математический анализ"),
    ).toBeInTheDocument();
    expect(screen.getByText("1 ч 40 мин")).toBeInTheDocument();
    expect(screen.getByText("Физика")).toBeInTheDocument();
    expect(screen.getByText("50 мин")).toBeInTheDocument();
  });

  it("рисует карту активности за тридцать недель", async () => {
    backend();
    render(<Stats />);

    expect(
      await screen.findByRole("img", { name: /Активность с 15 августа/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("30 недель")).toBeInTheDocument();
  });

  it("по умолчанию открыт период «неделя», и его можно сменить", async () => {
    backend();
    render(<Stats />);
    await screen.findByText("2 ч 30 мин");

    expect(callsOf("stats_time")[0][1]).toEqual({ range: "week" });

    await userEvent.click(screen.getByRole("button", { name: "Месяц" }));

    await waitFor(() => {
      expect(callsOf("stats_time")).toHaveLength(2);
    });
    expect(callsOf("stats_time")[1][1]).toEqual({ range: "month" });
  });

  it("предлагает повторить, если команда отказала", async () => {
    backend({ failing: "stats_time" });
    render(<Stats />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "база недоступна",
    );

    backend();
    await userEvent.click(screen.getByRole("button", { name: "Повторить" }));

    expect(await screen.findByText("2 ч 30 мин")).toBeInTheDocument();
  });

  it("не выдумывает данные, когда за период ничего не было", async () => {
    backend({
      time: {
        ...timeStats,
        total_seconds: 0,
        pomodoros: 0,
        streak_days: 0,
        subjects: [],
      },
    });
    render(<Stats />);

    expect(
      await screen.findByText("За этот период занятий не было"),
    ).toBeInTheDocument();
  });
});

describe("вкладка «Карточки»", () => {
  it("показывает ответы, верные и точность", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточки" }));

    expect(await screen.findByText("75 %")).toBeInTheDocument();
    expect(screen.getByText("40")).toBeInTheDocument();
    expect(screen.getByText("30")).toBeInTheDocument();
  });

  it("перечисляет проблемные карточки с их точностью", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточки" }));

    expect(
      await screen.findByText("Критерий обратимости матрицы"),
    ).toBeInTheDocument();
    expect(screen.getByText("2 из 10")).toBeInTheDocument();
    expect(screen.getByText("20%")).toBeInTheDocument();
  });

  it("ставит прочерк вместо точности, когда карточек не было", async () => {
    backend({
      cards: {
        ...cardsStats,
        answered: 0,
        correct: 0,
        accuracy_percent: 0,
        problems: [],
      },
    });
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточки" }));

    expect(await screen.findByText("—")).toBeInTheDocument();
    expect(
      screen.getByText("За этот период карточек не было"),
    ).toBeInTheDocument();
  });
});

describe("вкладка «Карточка»", () => {
  it("открывается на той карточке, по которой кликнули в проблемных", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточки" }));
    await screen.findByText("Критерий обратимости матрицы");

    await userEvent.click(
      screen.getByRole("button", { name: /Критерий обратимости матрицы/ }),
    );

    await waitFor(() => {
      expect(callsOf("stats_card")[0][1]).toEqual({ cardId: "c-1" });
    });
    expect(
      await screen.findByText("Определитель отличен от нуля"),
    ).toBeInTheDocument();
  });

  it("показывает историю ответов и среднее время припоминания", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточка" }));

    expect(await screen.findByText("в среднем 4,2 с")).toBeInTheDocument();
    expect(screen.getByText("Угадано 2 из 10.")).toBeInTheDocument();

    const answers = screen.getByRole("list");
    expect(within(answers).getAllByText("Не помню")).toHaveLength(2);
    expect(within(answers).getAllByText("Знаю")).toHaveLength(1);
  });

  it("сама выбирает первую карточку колоды, если её не выбрали", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточка" }));

    await waitFor(() => {
      expect(callsOf("stats_card")[0][1]).toEqual({ cardId: "c-1" });
    });
  });

  it("даёт выбрать другую карточку колоды", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточка" }));
    await screen.findByText("Угадано 2 из 10.");

    await userEvent.selectOptions(
      screen.getByLabelText("Карточка"),
      "Признак Даламбера",
    );

    await waitFor(() => {
      expect(callsOf("stats_card")[1][1]).toEqual({ cardId: "c-2" });
    });
  });

  it("периода на этой вкладке нет: история карточки — вся её история", async () => {
    backend();
    render(<Stats />);
    await userEvent.click(screen.getByRole("button", { name: "Карточка" }));

    await waitFor(() => {
      expect(
        screen.queryByRole("group", { name: "Период" }),
      ).not.toBeInTheDocument();
    });
  });
});

describe("экспорт", () => {
  it("отдаёт период таблицей за выбранный период", async () => {
    backend();
    render(<Stats />);
    await screen.findByText("2 ч 30 мин");

    await userEvent.click(
      screen.getByRole("button", { name: "Экспорт в CSV" }),
    );

    expect(await screen.findByLabelText("Таблица")).toHaveValue(
      "день,секунды\n2026-08-21,9000\n",
    );
    expect(callsOf("stats_export_csv")[0][1]).toEqual({ range: "week" });
  });
});
