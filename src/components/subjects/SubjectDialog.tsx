import { useState, type FormEvent } from "react";

import { Button, Dialog, Input } from "@/components/ui";
import {
  SUBJECT_COLORS,
  subjectBackground,
} from "@/components/subjects/palette";
import { cn } from "@/lib/cn";
import {
  createSubject,
  deleteSubject,
  errorMessage,
  updateSubject,
  type Subject,
  type SubjectInput,
} from "@/lib/tauri";

type SubjectDialogProps = {
  open: boolean;
  /** `null` — создание нового предмета. */
  subject: Subject | null;
  onClose: () => void;
  /** Вызывается после успешного сохранения или удаления. */
  onSaved: () => void;
};

/**
 * Диалог создания и правки предмета.
 *
 * Проверяет ввод не сам: пустое имя и незнакомый цвет отклоняет Rust, и его
 * сообщение показывается как есть. Дублировать правила в двух местах — верный
 * способ разойтись с ними при первом же изменении.
 */
export function SubjectDialog({
  open,
  subject,
  onClose,
  onSaved,
}: SubjectDialogProps) {
  // Форма заполняется один раз, при монтировании: экран монтирует диалог
  // в момент открытия, поэтому синхронизировать её с пропсами не нужно —
  // а если бы она синхронизировалась, обновление списка за спиной у диалога
  // затирало бы набранное.
  const [name, setName] = useState(subject?.name ?? "");
  const [color, setColor] = useState<string | null>(subject?.color ?? null);
  const [icon, setIcon] = useState(subject?.icon ?? "");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const input: SubjectInput = {
    name,
    color,
    icon: icon.trim() === "" ? null : icon,
  };

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);

    try {
      if (subject) {
        await updateSubject(subject.id, input);
      } else {
        await createSubject(input);
      }
      onSaved();
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
    } finally {
      setSaving(false);
    }
  }

  async function remove() {
    if (!subject) return;

    setSaving(true);
    setError(null);

    try {
      await deleteSubject(subject.id);
      onSaved();
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setSaving(false);
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={subject ? "Предмет" : "Новый предмет"}
      description={
        subject
          ? "Записанное время останется за предметом, даже если его удалить."
          : "Цвет можно не выбирать — предмет возьмёт следующий свободный."
      }
    >
      <form className="flex flex-col gap-4.5" onSubmit={submit}>
        <Input
          label="Название"
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Математический анализ"
          autoFocus
        />

        <fieldset className="flex flex-col gap-2.5">
          <legend className="text-11 tracking-label text-text-faint uppercase">
            Цвет
          </legend>
          <div className="flex flex-wrap gap-2.5">
            {SUBJECT_COLORS.map((slug) => (
              <button
                key={slug}
                type="button"
                aria-label={`Цвет ${slug}`}
                aria-pressed={color === slug}
                onClick={() => setColor(slug)}
                className={cn(
                  "flex size-11 items-center justify-center rounded-lg border transition-colors duration-150 ease-standard",
                  "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                  color === slug ? "border-accent" : "border-transparent",
                )}
              >
                <span
                  className={cn("size-6 rounded-md", subjectBackground(slug))}
                />
              </button>
            ))}
          </div>
        </fieldset>

        <Input
          label="Иконка"
          value={icon}
          onChange={(event) => setIcon(event.target.value)}
          placeholder="∫"
          maxLength={4}
          hint="Один символ или эмодзи. Можно оставить пустым."
        />

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}

        <div className="flex flex-col gap-2.5 sm:flex-row sm:justify-end">
          {subject &&
            (confirmingDelete ? (
              <Button
                variant="danger"
                onClick={remove}
                disabled={saving}
                className="sm:mr-auto"
              >
                Точно удалить
              </Button>
            ) : (
              <Button
                variant="danger"
                onClick={() => setConfirmingDelete(true)}
                disabled={saving}
                className="sm:mr-auto"
              >
                Удалить
              </Button>
            ))}
          <Button variant="secondary" onClick={onClose} disabled={saving}>
            Отмена
          </Button>
          <Button type="submit" variant="primary" disabled={saving}>
            Сохранить
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
