import { useState, type FormEvent } from "react";

import { Button, Dialog, Input, Select, Switch } from "@/components/ui";
import {
  createPreset,
  deletePreset,
  errorMessage,
  updatePreset,
  type Preset,
  type PresetInput,
  type PresetMode,
  type Subject,
} from "@/lib/tauri";

const MODE_LABELS: Record<PresetMode, string> = {
  countup: "Секундомер",
  countdown: "Обратный отсчёт",
  pomodoro: "Помодоро",
};

/** Минуты из поля ввода → секунды для команды. Пустое поле — это `null`. */
function toSeconds(minutes: string): number | null {
  const trimmed = minutes.trim();
  if (trimmed === "") return null;

  const value = Number(trimmed);
  return Number.isFinite(value) ? Math.round(value * 60) : null;
}

/** Секунды из БД → минуты для поля ввода. */
function toMinutes(seconds: number | null): string {
  if (seconds === null) return "";
  return String(seconds / 60);
}

type PresetDialogProps = {
  open: boolean;
  /** `null` — создание нового пресета. */
  preset: Preset | null;
  subjects: Subject[];
  onClose: () => void;
  onSaved: () => void;
};

/**
 * Диалог создания и правки пресета таймера.
 *
 * Поля показываются по режиму: у секундомера нет длительностей, у обратного
 * отсчёта — только рабочая. Значения незанятых полей сохраняются в форме,
 * пока диалог открыт, но в БД не уезжают: их отбрасывает `core::preset`.
 */
export function PresetDialog({
  open,
  preset,
  subjects,
  onClose,
  onSaved,
}: PresetDialogProps) {
  // Заполняется один раз, при монтировании — см. примечание в `SubjectDialog`.
  // Значения нового пресета — классические 25/5/15×4, а не пустые поля:
  // студенту остаётся согласиться, а не заполнять форму с нуля.
  const [name, setName] = useState(preset?.name ?? "");
  const [mode, setMode] = useState<PresetMode>(preset?.mode ?? "pomodoro");
  const [subjectId, setSubjectId] = useState(preset?.subject_id ?? "");
  const [work, setWork] = useState(
    preset ? toMinutes(preset.work_seconds) : "25",
  );
  const [rest, setRest] = useState(
    preset ? toMinutes(preset.break_seconds) : "5",
  );
  const [longRest, setLongRest] = useState(
    preset ? toMinutes(preset.long_break_seconds) : "15",
  );
  const [cycles, setCycles] = useState(
    preset?.cycles_before_long?.toString() ?? "4",
  );
  const [autoStart, setAutoStart] = useState(preset?.auto_start_next ?? false);
  const [isDefault, setIsDefault] = useState(preset?.is_default ?? false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const input: PresetInput = {
    subject_id: subjectId === "" ? null : subjectId,
    name,
    mode,
    work_seconds: toSeconds(work) ?? 0,
    break_seconds: toSeconds(rest),
    long_break_seconds: toSeconds(longRest),
    cycles_before_long: cycles.trim() === "" ? null : Number(cycles),
    auto_start_next: autoStart,
    is_default: isDefault,
  };

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);

    try {
      if (preset) {
        await updatePreset(preset.id, input);
      } else {
        await createPreset(input);
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
    if (!preset) return;

    setSaving(true);
    setError(null);

    try {
      await deletePreset(preset.id);
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
      title={preset ? "Пресет" : "Новый пресет"}
      description="Пресет без предмета доступен всем предметам сразу."
    >
      <form className="flex flex-col gap-4.5" onSubmit={submit}>
        <Input
          label="Название"
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Классический"
          autoFocus
        />

        <Select
          label="Режим"
          value={mode}
          onChange={(event) => setMode(event.target.value as PresetMode)}
        >
          {(Object.keys(MODE_LABELS) as PresetMode[]).map((value) => (
            <option key={value} value={value}>
              {MODE_LABELS[value]}
            </option>
          ))}
        </Select>

        <Select
          label="Предмет"
          value={subjectId}
          onChange={(event) => setSubjectId(event.target.value)}
        >
          <option value="">Все предметы</option>
          {subjects.map((subject) => (
            <option key={subject.id} value={subject.id}>
              {subject.name}
            </option>
          ))}
        </Select>

        {mode !== "countup" && (
          <Input
            label="Работа, минут"
            type="number"
            inputMode="numeric"
            min={1}
            value={work}
            onChange={(event) => setWork(event.target.value)}
          />
        )}

        {mode === "pomodoro" && (
          <>
            <div className="grid gap-4.5 sm:grid-cols-2">
              <Input
                label="Перерыв, минут"
                type="number"
                inputMode="numeric"
                min={1}
                value={rest}
                onChange={(event) => setRest(event.target.value)}
              />
              <Input
                label="Длинный перерыв, минут"
                type="number"
                inputMode="numeric"
                min={1}
                value={longRest}
                onChange={(event) => setLongRest(event.target.value)}
              />
            </div>

            <Input
              label="Циклов до длинного перерыва"
              type="number"
              inputMode="numeric"
              min={1}
              value={cycles}
              onChange={(event) => setCycles(event.target.value)}
            />

            <Switch
              label="Сразу начинать следующую фазу"
              checked={autoStart}
              onChange={setAutoStart}
            />
          </>
        )}

        <Switch
          label="Пресет по умолчанию"
          checked={isDefault}
          onChange={setIsDefault}
        />

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}

        <div className="flex flex-col gap-2.5 sm:flex-row sm:justify-end">
          {preset &&
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
