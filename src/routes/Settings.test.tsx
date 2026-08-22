import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Settings } from "@/routes/Settings";
import type {
  AdaptiveSettings,
  BlitzSettings,
  DaySettings,
  StreakSettings,
  ZenSettings,
} from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const defaults: ZenSettings = {
  minutes_only: false,
  font_size: "normal",
  dim_when_idle: true,
};
const midnight: DaySettings = { start_offset_seconds: 0 };
const twentySeconds: BlitzSettings = { seconds: 20 };
const middleOfTheSlider: AdaptiveSettings = { aggressiveness: 50 };
const tenMinutes: StreakSettings = { min_seconds: 600 };

/** Настройки, которые бэкенд помнит между вызовами. */
function backend(
  options: {
    stored?: ZenSettings;
    storedDay?: DaySettings;
    storedBlitz?: BlitzSettings;
    storedAdaptive?: AdaptiveSettings;
    storedStreak?: StreakSettings;
    saveFails?: boolean;
  } = {},
) {
  let stored = options.stored ?? defaults;
  let storedDay = options.storedDay ?? midnight;
  let storedBlitz = options.storedBlitz ?? twentySeconds;
  let storedAdaptive = options.storedAdaptive ?? middleOfTheSlider;
  let storedStreak = options.storedStreak ?? tenMinutes;

  invoke.mockImplementation((command: string, args?: unknown) => {
    if (options.saveFails && command.startsWith("set_")) {
      return Promise.reject({ kind: "database", message: "база недоступна" });
    }

    switch (command) {
      case "zen_settings":
        return Promise.resolve(stored);
      case "set_zen_settings": {
        const { minutesOnly, fontSize, dimWhenIdle } = args as {
          minutesOnly: boolean;
          fontSize: ZenSettings["font_size"];
          dimWhenIdle: boolean;
        };
        stored = {
          minutes_only: minutesOnly,
          font_size: fontSize,
          dim_when_idle: dimWhenIdle,
        };
        return Promise.resolve(stored);
      }
      case "day_settings":
        return Promise.resolve(storedDay);
      case "blitz_settings":
        return Promise.resolve(storedBlitz);
      case "set_blitz_settings": {
        const { seconds } = args as { seconds: number };
        storedBlitz = { seconds };
        return Promise.resolve(storedBlitz);
      }
      case "streak_settings":
        return Promise.resolve(storedStreak);
      case "set_streak_settings": {
        const { minSeconds } = args as { minSeconds: number };
        storedStreak = { min_seconds: minSeconds };
        return Promise.resolve(storedStreak);
      }
      case "adaptive_settings":
        return Promise.resolve(storedAdaptive);
      case "set_adaptive_settings": {
        const { aggressiveness } = args as { aggressiveness: number };
        storedAdaptive = { aggressiveness };
        return Promise.resolve(storedAdaptive);
      }
      case "set_day_settings": {
        const { startOffsetSeconds } = args as { startOffsetSeconds: number };
        storedDay = { start_offset_seconds: startOffsetSeconds };
        return Promise.resolve(storedDay);
      }
      default:
        return Promise.reject(new Error(`неожиданная команда ${command}`));
    }
  });
}

beforeEach(() => {
  invoke.mockReset();
});

describe("настройки", () => {
  it("показывает сохранённые значения чёрного экрана", async () => {
    backend({
      stored: {
        minutes_only: true,
        font_size: "large",
        dim_when_idle: false,
      },
    });
    render(<Settings />);

    const toggle = await screen.findByRole("switch", {
      name: "Показывать только минуты",
    });
    expect(toggle).toHaveAttribute("aria-checked", "true");
    expect(screen.getByLabelText("Размер цифр")).toHaveValue("large");
    expect(
      screen.getByRole("switch", { name: "Гасить экран без движения" }),
    ).toHaveAttribute("aria-checked", "false");
  });

  it("сохраняет переключатель «только минуты»", async () => {
    backend();
    render(<Settings />);
    const toggle = await screen.findByRole("switch", {
      name: "Показывать только минуты",
    });

    await userEvent.click(toggle);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_zen_settings", {
        minutesOnly: true,
        fontSize: "normal",
        dimWhenIdle: true,
      }),
    );
    expect(toggle).toHaveAttribute("aria-checked", "true");
  });

  it("сохраняет выбранный размер цифр", async () => {
    backend();
    render(<Settings />);
    const select = await screen.findByLabelText("Размер цифр");

    await userEvent.selectOptions(select, "small");

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_zen_settings", {
        minutesOnly: false,
        fontSize: "small",
        dimWhenIdle: true,
      }),
    );
    expect(select).toHaveValue("small");
  });

  it("сохраняет переключатель затемнения", async () => {
    backend();
    render(<Settings />);
    const toggle = await screen.findByRole("switch", {
      name: "Гасить экран без движения",
    });
    expect(toggle).toHaveAttribute("aria-checked", "true");

    await userEvent.click(toggle);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_zen_settings", {
        minutesOnly: false,
        fontSize: "normal",
        dimWhenIdle: false,
      }),
    );
    expect(toggle).toHaveAttribute("aria-checked", "false");
  });

  it("если запись не удалась, возвращает переключатель как было", async () => {
    backend({ saveFails: true });
    render(<Settings />);
    const toggle = await screen.findByRole("switch", {
      name: "Показывать только минуты",
    });

    await userEvent.click(toggle);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "база недоступна",
    );
    expect(toggle).toHaveAttribute("aria-checked", "false");
  });

  it("показывает сохранённую границу учебного дня", async () => {
    backend({ storedDay: { start_offset_seconds: 4 * 60 * 60 } });
    render(<Settings />);

    expect(await screen.findByLabelText("Начало учебного дня")).toHaveValue(
      String(4 * 60 * 60),
    );
  });

  it("сохраняет выбранную границу учебного дня", async () => {
    backend();
    render(<Settings />);
    const select = await screen.findByLabelText("Начало учебного дня");

    await userEvent.selectOptions(select, String(5 * 60 * 60));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_day_settings", {
        startOffsetSeconds: 5 * 60 * 60,
      }),
    );
    expect(select).toHaveValue(String(5 * 60 * 60));
  });

  it("если границу записать не удалось, возвращает прежнюю", async () => {
    backend({
      storedDay: { start_offset_seconds: 4 * 60 * 60 },
      saveFails: true,
    });
    render(<Settings />);
    const select = await screen.findByLabelText("Начало учебного дня");

    await userEvent.selectOptions(select, String(9 * 60 * 60));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "база недоступна",
    );
    expect(select).toHaveValue(String(4 * 60 * 60));
  });

  it("показывает перекос подбора словами, а не процентами", async () => {
    backend({ storedAdaptive: { aggressiveness: 100 } });
    render(<Settings />);

    const slider = await screen.findByLabelText("Перекос в сторону слабых");
    expect(slider).toHaveValue("100");
    expect(screen.getByText("Только слабые")).toBeInTheDocument();
  });

  it("сохраняет сдвинутый перекос подбора", async () => {
    backend();
    render(<Settings />);
    const slider = await screen.findByLabelText("Перекос в сторону слабых");

    fireEvent.change(slider, { target: { value: "0" } });

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_adaptive_settings", {
        aggressiveness: 0,
      }),
    );
    expect(slider).toHaveValue("0");
    expect(screen.getByText("Поровну")).toBeInTheDocument();
  });

  it("возвращает ползунок на место, если сохранить не удалось", async () => {
    backend({ storedAdaptive: { aggressiveness: 50 }, saveFails: true });
    render(<Settings />);
    const slider = await screen.findByLabelText("Перекос в сторону слабых");

    fireEvent.change(slider, { target: { value: "85" } });

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "база недоступна",
    );
    expect(slider).toHaveValue("50");
  });

  it("сохраняет дневной минимум серии", async () => {
    backend();
    render(<Settings />);
    const select = await screen.findByLabelText("Минимум за день");

    expect(select).toHaveValue("600");
    await userEvent.selectOptions(select, String(30 * 60));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_streak_settings", {
        minSeconds: 1800,
      }),
    );
    expect(select).toHaveValue("1800");
  });

  it("сохраняет время карточки в блице", async () => {
    backend();
    render(<Settings />);
    const select = await screen.findByLabelText("Время на карточку");

    await userEvent.selectOptions(select, "45");

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_blitz_settings", {
        seconds: 45,
      }),
    );
    expect(select).toHaveValue("45");
  });
});
