import { useEffect, useState } from "react";

import { Screen } from "@/components/Screen";
import { Card, Select, Slider, Switch } from "@/components/ui";
import {
  adaptiveSettings,
  blitzSettings,
  daySettings,
  errorMessage,
  saveAdaptiveSettings,
  saveBlitzSettings,
  saveDaySettings,
  saveStreakSettings,
  saveZenSettings,
  streakSettings,
  zenSettings,
  type AdaptiveSettings,
  type BlitzSettings,
  type DaySettings,
  type StreakSettings,
  type ZenFontSize,
  type ZenSettings,
} from "@/lib/tauri";

const FONT_SIZES: { value: ZenFontSize; label: string }[] = [
  { value: "small", label: "Мельче" },
  { value: "normal", label: "Обычный" },
  { value: "large", label: "Крупнее" },
];

/** Сколько секунд даётся на карточку в блице. */
const BLITZ_SECONDS = [10, 15, 20, 30, 45, 60];

/** Дневной минимум серии, в минутах. */
const STREAK_MINUTES = [5, 10, 15, 20, 30, 45, 60, 90, 120];

/**
 * Словами — то, что ползунок делает с подбором. Проценты сами по себе ничего
 * не говорят: важно, «поровну» или «в основном слабые».
 */
function leanLabel(aggressiveness: number): string {
  if (aggressiveness <= 10) return "Поровну";
  if (aggressiveness <= 35) return "Слегка";
  if (aggressiveness <= 65) return "Заметно";
  if (aggressiveness <= 90) return "Сильно";
  return "Только слабые";
}

/** Начало учебного дня выбирается по часам: минуты здесь ничего не решают. */
const DAY_START_HOURS = Array.from({ length: 24 }, (_, hour) => ({
  seconds: hour * 60 * 60,
  label: `${String(hour).padStart(2, "0")}:00`,
}));

/**
 * Раздел «Настройки»: граница учебного дня, блиц и чёрный экран.
 *
 * Пресеты таймера сюда не переезжают — они живут на экране «Таймеры», рядом
 * с предметами, которым принадлежат.
 */
export function Settings() {
  const [zen, setZen] = useState<ZenSettings | null>(null);
  const [day, setDay] = useState<DaySettings | null>(null);
  const [blitz, setBlitz] = useState<BlitzSettings | null>(null);
  const [adaptive, setAdaptive] = useState<AdaptiveSettings | null>(null);
  const [streak, setStreak] = useState<StreakSettings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      zenSettings(),
      daySettings(),
      blitzSettings(),
      adaptiveSettings(),
      streakSettings(),
    ])
      .then(
        ([loadedZen, loadedDay, loadedBlitz, loadedAdaptive, loadedStreak]) => {
          if (cancelled) return;
          setZen(loadedZen);
          setDay(loadedDay);
          setBlitz(loadedBlitz);
          setAdaptive(loadedAdaptive);
          setStreak(loadedStreak);
        },
      )
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  /**
   * Сохраняет и показывает выбранное сразу, не дожидаясь ответа: переключатель,
   * который «залипает» до конца запроса, ощущается сломанным. Если запись
   * не удалась, экран возвращается к тому, что действительно лежит в базе.
   */
  function saveZen(next: ZenSettings) {
    const previous = zen;
    setZen(next);
    setError(null);

    saveZenSettings(next)
      .then(setZen)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setZen(previous);
      });
  }

  function saveDay(startOffsetSeconds: number) {
    const previous = day;
    setDay({ start_offset_seconds: startOffsetSeconds });
    setError(null);

    saveDaySettings(startOffsetSeconds)
      .then(setDay)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setDay(previous);
      });
  }

  function saveBlitz(seconds: number) {
    const previous = blitz;
    setBlitz({ seconds });
    setError(null);

    saveBlitzSettings(seconds)
      .then(setBlitz)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setBlitz(previous);
      });
  }

  function saveAdaptive(aggressiveness: number) {
    const previous = adaptive;
    setAdaptive({ aggressiveness });
    setError(null);

    saveAdaptiveSettings(aggressiveness)
      .then(setAdaptive)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setAdaptive(previous);
      });
  }

  function saveStreak(minSeconds: number) {
    const previous = streak;
    setStreak({ min_seconds: minSeconds });
    setError(null);

    saveStreakSettings(minSeconds)
      .then(setStreak)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setStreak(previous);
      });
  }

  return (
    <Screen title="Настройки">
      <Card title="Учебный день">
        {day ? (
          <Select
            label="Начало учебного дня"
            hint="Время до этой границы засчитывается в предыдущий день — чтобы ночная сессия не разрывалась пополам. Записи при этом не удаляются никогда."
            value={String(day.start_offset_seconds)}
            onChange={(event) => saveDay(Number(event.target.value))}
          >
            {DAY_START_HOURS.map((hour) => (
              <option key={hour.seconds} value={hour.seconds}>
                {hour.label}
              </option>
            ))}
          </Select>
        ) : (
          <p className="text-14 text-text-dim">
            {error ? "Настройки не прочитались" : "Загрузка…"}
          </p>
        )}
      </Card>

      <Card title="Серия">
        {streak ? (
          <Select
            label="Минимум за день"
            hint="Сколько нужно позаниматься, чтобы день попал в серию. Границу дня серия берёт ту же, что и всё остальное, — из настройки выше."
            value={String(streak.min_seconds)}
            onChange={(event) => saveStreak(Number(event.target.value))}
          >
            {STREAK_MINUTES.map((minutes) => (
              <option key={minutes} value={minutes * 60}>
                {minutes} мин
              </option>
            ))}
          </Select>
        ) : (
          <p className="text-14 text-text-dim">
            {error ? "Настройки не прочитались" : "Загрузка…"}
          </p>
        )}
      </Card>

      <Card title="Блиц">
        {blitz ? (
          <Select
            label="Время на карточку"
            hint="Когда время выходит, карточка засчитывается как «не помню» — блиц про скорость припоминания, а не про раздумья."
            value={String(blitz.seconds)}
            onChange={(event) => saveBlitz(Number(event.target.value))}
          >
            {BLITZ_SECONDS.map((seconds) => (
              <option key={seconds} value={seconds}>
                {seconds} с
              </option>
            ))}
          </Select>
        ) : (
          <p className="text-14 text-text-dim">
            {error ? "Настройки не прочитались" : "Загрузка…"}
          </p>
        )}
      </Card>

      <Card title="Карточки">
        {adaptive ? (
          <Slider
            label="Перекос в сторону слабых"
            hint="Внутри любого режима слабые карточки выпадают чаще, а знакомые реже — но из оборота не выходит ни одна. Слева — обычное перемешивание, справа — заход почти целиком из того, что не даётся."
            min={0}
            max={100}
            step={5}
            value={adaptive.aggressiveness}
            valueLabel={leanLabel(adaptive.aggressiveness)}
            onChange={(event) => saveAdaptive(Number(event.target.value))}
          />
        ) : (
          <p className="text-14 text-text-dim">
            {error ? "Настройки не прочитались" : "Загрузка…"}
          </p>
        )}
      </Card>

      <Card title="Чёрный экран">
        {zen ? (
          <>
            <Switch
              label="Показывать только минуты"
              checked={zen.minutes_only}
              onChange={(minutes_only) => saveZen({ ...zen, minutes_only })}
            />
            <Switch
              label="Гасить экран без движения"
              checked={zen.dim_when_idle}
              onChange={(dim_when_idle) => saveZen({ ...zen, dim_when_idle })}
            />
            <Select
              label="Размер цифр"
              value={zen.font_size}
              onChange={(event) =>
                saveZen({
                  ...zen,
                  font_size: event.target.value as ZenFontSize,
                })
              }
            >
              {FONT_SIZES.map((size) => (
                <option key={size.value} value={size.value}>
                  {size.label}
                </option>
              ))}
            </Select>
          </>
        ) : (
          <p className="text-14 text-text-dim">
            {error ? "Настройки не прочитались" : "Загрузка…"}
          </p>
        )}
      </Card>

      {error && (
        <p className="text-13 text-danger-text" role="alert">
          {error}
        </p>
      )}
    </Screen>
  );
}
