import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Settings } from "@/routes/Settings";
import type { ZenSettings } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const defaults: ZenSettings = { minutes_only: false, font_size: "normal" };

/** Настройки, которые бэкенд помнит между вызовами. */
function backend(options: { stored?: ZenSettings; saveFails?: boolean } = {}) {
  let stored = options.stored ?? defaults;

  invoke.mockImplementation((command: string, args?: unknown) => {
    switch (command) {
      case "zen_settings":
        return Promise.resolve(stored);
      case "set_zen_settings": {
        if (options.saveFails) {
          return Promise.reject({
            kind: "database",
            message: "база недоступна",
          });
        }
        const { minutesOnly, fontSize } = args as {
          minutesOnly: boolean;
          fontSize: ZenSettings["font_size"];
        };
        stored = { minutes_only: minutesOnly, font_size: fontSize };
        return Promise.resolve(stored);
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
    backend({ stored: { minutes_only: true, font_size: "large" } });
    render(<Settings />);

    const toggle = await screen.findByRole("switch", {
      name: "Показывать только минуты",
    });
    expect(toggle).toHaveAttribute("aria-checked", "true");
    expect(screen.getByLabelText("Размер цифр")).toHaveValue("large");
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
      }),
    );
    expect(select).toHaveValue("small");
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
});
