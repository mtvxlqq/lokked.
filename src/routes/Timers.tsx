import { useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { Screen } from "@/components/Screen";
import { PresetDialog } from "@/components/presets/PresetDialog";
import { PresetList } from "@/components/presets/PresetList";
import { SubjectDialog } from "@/components/subjects/SubjectDialog";
import { SubjectList } from "@/components/subjects/SubjectList";
import { TodaySummary } from "@/components/today/TodaySummary";
import { Button, Card, EmptyState } from "@/components/ui";
import { TimerIcon } from "@/components/nav/icons";
import {
  errorMessage,
  listPresets,
  listSubjects,
  todayTotals,
  type Preset,
  type Subject,
  type TodayTotals,
} from "@/lib/tauri";

/** Что показывает экран, пока команды не ответили или если они отказали. */
type LoadState = "loading" | "ready" | "failed";

/**
 * Раздел «Таймеры» — сводка за учебный день, предметы и пресеты таймера.
 *
 * Данных экран не придумывает: и список, и суммы приходят из tauri-команд.
 *
 * Учебный день кончается не тогда, когда о нём вспомнят: экран сам заводит
 * будильник на ближайшую границу и перечитывает сводку, когда она наступит.
 * Ничего при этом не удаляется — меняется только день, по которому идёт
 * фильтр.
 */
export function Timers() {
  const navigate = useNavigate();

  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [today, setToday] = useState<TodayTotals | null>(null);
  const [secondsToday, setSecondsToday] = useState(new Map<string, number>());
  const [state, setState] = useState<LoadState>("loading");
  const [error, setError] = useState<string | null>(null);

  const [subjectDialog, setSubjectDialog] = useState<{
    open: boolean;
    subject: Subject | null;
  }>({ open: false, subject: null });
  const [presetDialog, setPresetDialog] = useState<{
    open: boolean;
    preset: Preset | null;
  }>({ open: false, preset: null });

  // Перезагрузка выражена данными, а не вызовом: диалог сохранил предмет —
  // счётчик растёт, эффект отрабатывает заново. Так запрос остаётся внутри
  // эффекта, вместе с отменой на размонтировании.
  const [reloads, setReloads] = useState(0);
  const reload = () => setReloads((count) => count + 1);

  useEffect(() => {
    let cancelled = false;

    Promise.all([listSubjects(), listPresets(), todayTotals()])
      .then(([loadedSubjects, loadedPresets, totals]) => {
        if (cancelled) return;

        setSubjects(loadedSubjects);
        setPresets(loadedPresets);
        setToday(totals);
        setSecondsToday(new Map(totals.seconds_by_subject));
        setError(null);
        setState("ready");
      })
      .catch((failure: unknown) => {
        if (cancelled) return;

        setError(errorMessage(failure));
        setState("failed");
      });

    return () => {
      cancelled = true;
    };
  }, [reloads]);

  // Будильник на смену учебного дня. Момент границы считает бэкенд, поэтому
  // здесь не надо знать ни про часовой пояс, ни про настройку — только про то,
  // что после неё сводку надо спросить заново.
  useEffect(() => {
    if (!today) return;

    const wait = new Date(today.next_boundary).getTime() - Date.now();
    // Секунда сверху: спрашивать ровно в момент границы — значит рисковать
    // получить ответ за старый день из-за расхождения часов на миллисекунды.
    const id = setTimeout(reload, Math.max(0, wait) + 1000);

    return () => clearTimeout(id);
  }, [today]);

  // Возвращение из фона: окно могло провисеть свёрнутым всю ночь, а таймеры
  // в это время не срабатывали. Перечитываем, не дожидаясь границы.
  useEffect(() => {
    function onVisibility() {
      if (!document.hidden) reload();
    }

    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  return (
    <Screen
      title="Таймеры"
      actions={
        state === "ready" &&
        subjects.length > 0 && (
          <Button
            variant="primary"
            size="sm"
            onClick={() => setSubjectDialog({ open: true, subject: null })}
          >
            Новый предмет
          </Button>
        )
      }
    >
      {state === "loading" && (
        <p className="text-14 text-text-dim">Загрузка…</p>
      )}

      {state === "failed" && (
        <Card title="Не удалось загрузить данные">
          <p className="text-14 text-danger-text" role="alert">
            {error}
          </p>
          <div>
            <Button variant="secondary" onClick={reload}>
              Повторить
            </Button>
          </div>
        </Card>
      )}

      {state === "ready" && today && subjects.length > 0 && (
        <TodaySummary totals={today} />
      )}

      {state === "ready" && subjects.length === 0 && (
        <EmptyState
          icon={<TimerIcon className="size-8" />}
          title="Предметов пока нет"
          description="Добавь первый предмет — с него и начнётся счёт времени."
          action={
            <Button
              variant="primary"
              onClick={() => setSubjectDialog({ open: true, subject: null })}
            >
              Добавить предмет
            </Button>
          }
        />
      )}

      {state === "ready" && subjects.length > 0 && (
        <Card title="Предметы" aside="время за сегодня">
          <SubjectList
            subjects={subjects}
            secondsToday={secondsToday}
            onStart={(subject) => void navigate(`/timer/${subject.id}`)}
            onEdit={(subject) => setSubjectDialog({ open: true, subject })}
          />
        </Card>
      )}

      {state === "ready" && (
        <Card title="Пресеты таймера">
          {presets.length === 0 ? (
            <p className="text-14 text-text-dim">
              Пресетов пока нет. Пресет задаёт режим и длительности, с которыми
              запускается сессия.
            </p>
          ) : (
            <PresetList
              presets={presets}
              subjects={subjects}
              onEdit={(preset) => setPresetDialog({ open: true, preset })}
            />
          )}
          <div>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setPresetDialog({ open: true, preset: null })}
            >
              Новый пресет
            </Button>
          </div>
        </Card>
      )}

      {/* Закрытый диалог не просто скрыт, а не существует: иначе его поля
          остаются в документе и попадают, например, в поиск по странице. */}
      {subjectDialog.open && (
        <SubjectDialog
          open
          subject={subjectDialog.subject}
          onClose={() => setSubjectDialog({ open: false, subject: null })}
          onSaved={reload}
        />
      )}

      {presetDialog.open && (
        <PresetDialog
          open
          preset={presetDialog.preset}
          subjects={subjects}
          onClose={() => setPresetDialog({ open: false, preset: null })}
          onSaved={reload}
        />
      )}
    </Screen>
  );
}
