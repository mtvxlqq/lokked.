import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PresetDialog } from "@/components/presets/PresetDialog";
import type { Preset, Subject } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const subjects: Subject[] = [
  { id: "s-1", name: "Алгебра", color: "subject-1", icon: null, position: 0 },
];

const pomodoro: Preset = {
  id: "p-1",
  subject_id: null,
  name: "Классический",
  mode: "pomodoro",
  work_seconds: 25 * 60,
  break_seconds: 5 * 60,
  long_break_seconds: 15 * 60,
  cycles_before_long: 4,
  auto_start_next: false,
  is_default: true,
};

describe("PresetDialog", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue(pomodoro);
  });

  it("создаёт помодоро с классическими значениями по умолчанию", async () => {
    const user = userEvent.setup();

    render(
      <PresetDialog
        open
        preset={null}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Название"), "Классический");
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(invoke).toHaveBeenCalledWith("create_preset", {
      input: {
        subject_id: null,
        name: "Классический",
        mode: "pomodoro",
        work_seconds: 25 * 60,
        break_seconds: 5 * 60,
        long_break_seconds: 15 * 60,
        cycles_before_long: 4,
        auto_start_next: false,
        is_default: false,
      },
    });
  });

  it("прячет длительности у секундомера", async () => {
    const user = userEvent.setup();

    render(
      <PresetDialog
        open
        preset={null}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.selectOptions(screen.getByLabelText("Режим"), "countup");

    expect(screen.queryByLabelText("Работа, минут")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Перерыв, минут")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("switch", { name: "Сразу начинать следующую фазу" }),
    ).not.toBeInTheDocument();
  });

  it("у обратного отсчёта оставляет только рабочую фазу", async () => {
    const user = userEvent.setup();

    render(
      <PresetDialog
        open
        preset={null}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.selectOptions(screen.getByLabelText("Режим"), "countdown");
    await user.type(screen.getByLabelText("Название"), "45 минут");

    const work = screen.getByLabelText("Работа, минут");
    await user.clear(work);
    await user.type(work, "45");
    expect(screen.queryByLabelText("Перерыв, минут")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(invoke).toHaveBeenCalledWith(
      "create_preset",
      expect.objectContaining({
        input: expect.objectContaining({
          mode: "countdown",
          work_seconds: 45 * 60,
        }),
      }),
    );
  });

  it("привязывает пресет к предмету", async () => {
    const user = userEvent.setup();

    render(
      <PresetDialog
        open
        preset={null}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Название"), "Алгебра — длинный");
    await user.selectOptions(screen.getByLabelText("Предмет"), "s-1");
    await user.click(
      screen.getByRole("switch", { name: "Пресет по умолчанию" }),
    );
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(invoke).toHaveBeenCalledWith(
      "create_preset",
      expect.objectContaining({
        input: expect.objectContaining({
          subject_id: "s-1",
          is_default: true,
        }),
      }),
    );
  });

  it("подставляет минуты существующего пресета", () => {
    render(
      <PresetDialog
        open
        preset={pomodoro}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    expect(screen.getByLabelText("Работа, минут")).toHaveValue(25);
    expect(screen.getByLabelText("Перерыв, минут")).toHaveValue(5);
    expect(screen.getByLabelText("Длинный перерыв, минут")).toHaveValue(15);
    expect(screen.getByLabelText("Циклов до длинного перерыва")).toHaveValue(4);
    expect(
      screen.getByRole("switch", { name: "Пресет по умолчанию" }),
    ).toHaveAttribute("aria-checked", "true");
  });

  it("показывает отказ команды", async () => {
    const user = userEvent.setup();
    invoke.mockRejectedValue({
      kind: "validation",
      message: "поле break_seconds должно быть больше нуля",
    });

    render(
      <PresetDialog
        open
        preset={pomodoro}
        subjects={subjects}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "поле break_seconds должно быть больше нуля",
    );
  });
});
