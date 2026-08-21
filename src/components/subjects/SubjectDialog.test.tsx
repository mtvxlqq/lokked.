import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SubjectDialog } from "@/components/subjects/SubjectDialog";
import type { Subject } from "@/lib/tauri";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const subject: Subject = {
  id: "s-1",
  name: "Алгебра",
  color: "subject-3",
  icon: null,
  position: 0,
};

describe("SubjectDialog", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue(subject);
  });

  it("создаёт предмет без выбранного цвета", async () => {
    const user = userEvent.setup();
    const onSaved = vi.fn();

    render(
      <SubjectDialog open subject={null} onClose={vi.fn()} onSaved={onSaved} />,
    );

    await user.type(screen.getByLabelText("Название"), "Алгебра");
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    // Цвет уходит как null: выбирать за студента будет бэкенд.
    expect(invoke).toHaveBeenCalledWith("create_subject", {
      input: { name: "Алгебра", color: null, icon: null },
    });
    expect(onSaved).toHaveBeenCalled();
  });

  it("передаёт выбранный цвет и иконку", async () => {
    const user = userEvent.setup();

    render(
      <SubjectDialog open subject={null} onClose={vi.fn()} onSaved={vi.fn()} />,
    );

    await user.type(screen.getByLabelText("Название"), "Физика");
    await user.click(screen.getByRole("button", { name: "Цвет subject-5" }));
    await user.type(screen.getByLabelText("Иконка"), "∫");
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(invoke).toHaveBeenCalledWith("create_subject", {
      input: { name: "Физика", color: "subject-5", icon: "∫" },
    });
  });

  it("подставляет поля существующего предмета и правит его", async () => {
    const user = userEvent.setup();

    render(
      <SubjectDialog
        open
        subject={subject}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    const name = screen.getByLabelText("Название");
    expect(name).toHaveValue("Алгебра");
    expect(
      screen.getByRole("button", { name: "Цвет subject-3" }),
    ).toHaveAttribute("aria-pressed", "true");

    await user.clear(name);
    await user.type(name, "Линейная алгебра");
    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(invoke).toHaveBeenCalledWith("update_subject", {
      id: "s-1",
      input: { name: "Линейная алгебра", color: "subject-3", icon: null },
    });
  });

  it("показывает отказ команды и не закрывает диалог", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    invoke.mockRejectedValue({
      kind: "validation",
      message: "название предмета не может быть пустым",
    });

    render(
      <SubjectDialog open subject={null} onClose={onClose} onSaved={vi.fn()} />,
    );

    await user.click(screen.getByRole("button", { name: "Сохранить" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "название предмета не может быть пустым",
    );
    expect(onClose).not.toHaveBeenCalled();
  });

  it("удаляет только после подтверждения", async () => {
    const user = userEvent.setup();

    render(
      <SubjectDialog
        open
        subject={subject}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Удалить" }));
    expect(invoke).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Точно удалить" }));
    expect(invoke).toHaveBeenCalledWith("delete_subject", { id: "s-1" });
  });
});
