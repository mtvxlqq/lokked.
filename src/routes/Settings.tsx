import { useEffect, useState } from "react";

import { Screen } from "@/components/Screen";
import { Card, Select, Switch } from "@/components/ui";
import {
  errorMessage,
  saveZenSettings,
  zenSettings,
  type ZenFontSize,
  type ZenSettings,
} from "@/lib/tauri";

const FONT_SIZES: { value: ZenFontSize; label: string }[] = [
  { value: "small", label: "Мельче" },
  { value: "normal", label: "Обычный" },
  { value: "large", label: "Крупнее" },
];

/**
 * Раздел «Настройки».
 *
 * Пока здесь только чёрный экран; граница учебного дня появится в M8, а
 * пресеты таймера живут на экране «Таймеры», рядом с предметами, которым они
 * принадлежат.
 */
export function Settings() {
  const [settings, setSettings] = useState<ZenSettings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    zenSettings()
      .then((loaded) => {
        if (!cancelled) setSettings(loaded);
      })
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
  function save(next: ZenSettings) {
    const previous = settings;
    setSettings(next);
    setError(null);

    saveZenSettings(next)
      .then(setSettings)
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setSettings(previous);
      });
  }

  return (
    <Screen title="Настройки">
      <Card title="Чёрный экран">
        {settings ? (
          <>
            <Switch
              label="Показывать только минуты"
              checked={settings.minutes_only}
              onChange={(minutes_only) => save({ ...settings, minutes_only })}
            />
            <Select
              label="Размер цифр"
              value={settings.font_size}
              onChange={(event) =>
                save({
                  ...settings,
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

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </Card>
    </Screen>
  );
}
