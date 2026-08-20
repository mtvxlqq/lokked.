import { useMemo } from "react";

/**
 * Страница сверки токенов с макетом. Только для разработки: маршрут
 * `/dev/tokens` регистрируется в `router.tsx` под `import.meta.env.DEV`,
 * в релизную сборку этот модуль не попадает.
 *
 * Ни одного значения цвета в коде страницы нет — свотчи читают вычисленное
 * значение переменной прямо из документа. Если токен переименуют или удалят,
 * страница покажет пустое значение, а не устаревший hex.
 */

function useCssVar(name: string): string {
  return useMemo(
    () =>
      getComputedStyle(document.documentElement).getPropertyValue(name).trim(),
    [name],
  );
}

type SwatchProps = {
  /** Имя токена без префикса пространства имён: `surface`, `accent-text`. */
  token: string;
  /** Утилита Tailwind, красящая образец: `bg-surface`, `bg-accent-text`. */
  className: string;
  /** Где токен применяется. */
  note?: string;
};

function Swatch({ token, className, note }: SwatchProps) {
  const value = useCssVar(`--color-${token}`);

  return (
    <div className="flex items-center gap-3">
      <span
        className={`size-11 shrink-0 rounded-sm border border-border ${className}`}
      />
      <span className="flex min-w-0 flex-col gap-0.5">
        <span className="truncate font-mono text-12.5 text-text-2">
          {token}
        </span>
        <span className="truncate font-mono text-11 text-text-faint uppercase">
          {value}
        </span>
        {note && <span className="truncate text-11 text-text-dim">{note}</span>}
      </span>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-4.5">
      <h2 className="font-mono text-12 tracking-label-2 text-accent-text-2 uppercase">
        {title}
      </h2>
      {children}
    </section>
  );
}

function Grid({ children }: { children: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-3.5">
      {children}
    </div>
  );
}

const SURFACES: SwatchProps[] = [
  { token: "bg", className: "bg-bg", note: "фон приложения" },
  { token: "bg-zen", className: "bg-bg-zen", note: "только Zen и шеринг" },
  { token: "surface", className: "bg-surface", note: "карточки, строки" },
  { token: "surface-sunken", className: "bg-surface-sunken", note: "сайдбар" },
  { token: "surface-inset", className: "bg-surface-inset", note: "формула" },
  { token: "raised", className: "bg-raised", note: "активный пункт нав." },
  { token: "raised-2", className: "bg-raised-2", note: "аватар" },
  { token: "row", className: "bg-row", note: "строка лидерборда" },
  { token: "track", className: "bg-track", note: "дорожка" },
  {
    token: "danger-surface",
    className: "bg-danger-surface",
    note: "блиц 0:05",
  },
  {
    token: "danger-surface-2",
    className: "bg-danger-surface-2",
    note: "на проверке",
  },
  {
    token: "accent-surface",
    className: "bg-accent-surface",
    note: "кнопка-призрак",
  },
  { token: "light-surface", className: "bg-light-surface", note: "на светлом" },
];

const BORDERS: SwatchProps[] = [
  { token: "border", className: "bg-border" },
  { token: "border-soft", className: "bg-border-soft" },
  { token: "hairline", className: "bg-hairline" },
  { token: "hairline-2", className: "bg-hairline-2" },
  { token: "border-strong", className: "bg-border-strong" },
  { token: "border-strong-2", className: "bg-border-strong-2" },
  { token: "border-mute", className: "bg-border-mute" },
  { token: "border-accent", className: "bg-border-accent" },
  { token: "border-danger", className: "bg-border-danger" },
];

const TEXT: SwatchProps[] = [
  { token: "text", className: "bg-text" },
  { token: "text-bright", className: "bg-text-bright", note: "цифры" },
  { token: "text-1", className: "bg-text-1" },
  { token: "text-2", className: "bg-text-2" },
  { token: "text-3", className: "bg-text-3" },
  { token: "text-4", className: "bg-text-4" },
  { token: "text-muted", className: "bg-text-muted" },
  { token: "text-muted-2", className: "bg-text-muted-2" },
  { token: "text-dim", className: "bg-text-dim" },
  { token: "text-dim-2", className: "bg-text-dim-2" },
  { token: "text-faint", className: "bg-text-faint" },
  { token: "text-disabled", className: "bg-text-disabled" },
  { token: "text-zen-dim", className: "bg-text-zen-dim", note: "Zen idle" },
  { token: "text-zen-dim-2", className: "bg-text-zen-dim-2" },
];

const ACCENT: SwatchProps[] = [
  { token: "accent", className: "bg-accent", note: "основной (Steel)" },
  { token: "accent-text", className: "bg-accent-text" },
  { token: "accent-text-2", className: "bg-accent-text-2" },
  { token: "accent-text-3", className: "bg-accent-text-3" },
  { token: "accent-on-light", className: "bg-accent-on-light" },
  { token: "accent-alt-teal", className: "bg-accent-alt-teal" },
  { token: "accent-alt-lilac", className: "bg-accent-alt-lilac" },
  { token: "danger", className: "bg-danger" },
  { token: "danger-bright", className: "bg-danger-bright", note: "0:05" },
  { token: "danger-text", className: "bg-danger-text" },
  { token: "danger-text-2", className: "bg-danger-text-2" },
];

const SUBJECTS: SwatchProps[] = [
  { token: "subject-1", className: "bg-subject-1" },
  { token: "subject-2", className: "bg-subject-2" },
  { token: "subject-3", className: "bg-subject-3" },
  { token: "subject-4", className: "bg-subject-4" },
  { token: "subject-5", className: "bg-subject-5" },
  { token: "subject-6", className: "bg-subject-6" },
  { token: "subject-7", className: "bg-subject-7" },
  { token: "subject-8", className: "bg-subject-8" },
];

const HEATMAP: SwatchProps[] = [
  { token: "heat-0", className: "bg-heat-0" },
  { token: "heat-1", className: "bg-heat-1" },
  { token: "heat-2", className: "bg-heat-2" },
  { token: "heat-3", className: "bg-heat-3" },
  { token: "heat-4", className: "bg-heat-4" },
];

const STREAK: { label: string; className: string }[] = [
  { label: "зачтено", className: "bg-streak-done text-streak-done-text" },
  { label: "заморозка", className: "bg-streak-frozen text-streak-frozen-text" },
  { label: "пропуск", className: "bg-streak-missed text-streak-missed-text" },
  { label: "будущий", className: "bg-streak-future text-streak-future-text" },
  {
    label: "сегодня",
    className: "bg-streak-future text-streak-done-text border border-accent",
  },
];

/** Полная шкала из tokens.md; порядок — от мелкого к крупному. */
const TYPE_SCALE = [
  "text-10.5",
  "text-11",
  "text-11.5",
  "text-12",
  "text-12.5",
  "text-13",
  "text-13.5",
  "text-14",
  "text-14.5",
  "text-15",
  "text-15.5",
  "text-16",
  "text-17",
  "text-18",
  "text-19",
  "text-20",
  "text-21",
  "text-22",
  "text-24",
  "text-26",
  "text-28",
  "text-30",
  "text-34",
  "text-38",
  "text-40",
  "text-42",
  "text-52",
  "text-58",
  "text-64",
  "text-76",
  "text-88",
  "text-104",
  "text-112",
  "text-150",
  "text-212",
];

const TRACKING = [
  "tracking-timer",
  "tracking-timer-2",
  "tracking-title",
  "tracking-tight",
  "tracking-normal",
  "tracking-label",
  "tracking-label-2",
  "tracking-label-3",
  "tracking-label-4",
  "tracking-wide",
  "tracking-wider",
  "tracking-widest",
  "tracking-zen-subject",
  "tracking-zen-subject-sm",
];

const RADII = [
  { name: "radius-xs", className: "rounded-xs" },
  { name: "radius-sm", className: "rounded-sm" },
  { name: "radius-md", className: "rounded-md" },
  { name: "radius-lg", className: "rounded-lg" },
  { name: "radius-xl", className: "rounded-xl" },
  { name: "radius-2xl", className: "rounded-2xl" },
  { name: "radius-icon", className: "rounded-icon" },
  { name: "radius-full", className: "rounded-full" },
];

const GLOWS = [
  { name: "glow-timer", className: "glow-timer text-58" },
  { name: "glow-timer-dim", className: "glow-timer-dim text-58" },
  { name: "glow-streak", className: "glow-streak text-58" },
  { name: "glow-today", className: "glow-today text-58" },
  { name: "glow-score", className: "glow-score text-58" },
  { name: "glow-danger", className: "glow-danger text-58" },
];

export function Tokens() {
  return (
    <main className="mx-auto flex max-w-app flex-col gap-8 px-5 py-6.5 sm:px-14 sm:py-11">
      <header className="flex flex-col gap-2.5">
        <h1 className="text-21 font-semibold tracking-title text-text sm:text-30">
          Дизайн-токены
        </h1>
        <p className="text-14 leading-text text-text-muted sm:text-15">
          Сверка с макетом. Значения читаются из вычисленного CSS, а не
          продублированы в коде страницы.
        </p>
      </header>

      <Section title="Поверхности">
        <Grid>
          {SURFACES.map((s) => (
            <Swatch key={s.token} {...s} />
          ))}
        </Grid>
      </Section>

      <Section title="Обводки">
        <Grid>
          {BORDERS.map((s) => (
            <Swatch key={s.token} {...s} />
          ))}
        </Grid>
      </Section>

      <Section title="Текст">
        <Grid>
          {TEXT.map((s) => (
            <Swatch key={s.token} {...s} />
          ))}
        </Grid>
      </Section>

      <Section title="Акцент и опасность">
        <Grid>
          {ACCENT.map((s) => (
            <Swatch key={s.token} {...s} />
          ))}
        </Grid>
      </Section>

      <Section title="Предметы">
        <Grid>
          {SUBJECTS.map((s) => (
            <Swatch key={s.token} {...s} />
          ))}
        </Grid>
      </Section>

      <Section title="Heatmap активности">
        <div className="flex flex-wrap gap-2">
          {HEATMAP.map((s) => (
            <span
              key={s.token}
              className={`size-8 rounded-xs ${s.className}`}
              title={s.token}
            />
          ))}
        </div>
      </Section>

      <Section title="Календарь стрика">
        <div className="flex flex-wrap gap-3">
          {STREAK.map((s) => (
            <span
              key={s.label}
              className={`flex h-11 min-w-11 items-center justify-center rounded-md px-3 font-mono text-13 ${s.className}`}
            >
              {s.label}
            </span>
          ))}
        </div>
      </Section>

      <Section title="Шрифты">
        <div className="flex flex-col gap-3.5">
          <p className="font-mono text-19 text-text-1">
            JetBrains Mono — 0123456789 Цифры Digits
          </p>
          <p className="font-sans text-19 text-text-1">
            IBM Plex Sans — интерфейс, Interface
          </p>
          <p className="font-math text-19 text-text-1 italic">
            Georgia — формулы, x² + y² = r²
          </p>
        </div>
      </Section>

      <Section title="Шкала размеров">
        <div className="flex flex-col gap-2 overflow-x-auto">
          {TYPE_SCALE.map((size) => (
            <div key={size} className="flex items-baseline gap-4">
              <span className="w-24 shrink-0 font-mono text-11 text-text-faint">
                {size}
              </span>
              <span
                className={`font-mono whitespace-nowrap text-text-3 ${size}`}
              >
                12:34
              </span>
            </div>
          ))}
        </div>
      </Section>

      <Section title="Трекинг">
        <div className="flex flex-col gap-2">
          {TRACKING.map((t) => (
            <div key={t} className="flex flex-wrap items-baseline gap-4">
              <span className="w-44 shrink-0 font-mono text-11 text-text-faint">
                {t}
              </span>
              <span className={`text-14 text-text-3 uppercase ${t}`}>
                Lokked
              </span>
            </div>
          ))}
        </div>
      </Section>

      <Section title="Радиусы">
        <div className="flex flex-wrap gap-4">
          {RADII.map((r) => (
            <div key={r.name} className="flex flex-col items-center gap-2">
              <span
                className={`size-16 border border-border-strong bg-surface ${r.className}`}
              />
              <span className="font-mono text-11 text-text-faint">
                {r.name}
              </span>
            </div>
          ))}
        </div>
      </Section>

      <Section title="Свечение">
        <div className="flex flex-col gap-4 rounded-xl bg-bg-zen p-7">
          {GLOWS.map((g) => (
            <div key={g.name} className="flex flex-wrap items-center gap-6">
              <span className="w-44 shrink-0 font-mono text-11 text-text-faint">
                {g.name}
              </span>
              <span className={`font-mono tabular-nums ${g.className}`}>
                15:47
              </span>
            </div>
          ))}
        </div>
        <div className="flex flex-wrap gap-4">
          <div className="halo-zen flex h-44 flex-1 items-center justify-center rounded-xl">
            <span className="glow-timer animate-breathe font-mono text-42 tabular-nums">
              1:12:04
            </span>
          </div>
          <div className="halo-zen-dim flex h-44 flex-1 items-center justify-center rounded-xl">
            <span className="glow-timer-dim animate-breathe-dim font-mono text-42 tabular-nums">
              1:12:04
            </span>
          </div>
        </div>
      </Section>

      <Section title="Тени">
        <div className="flex flex-wrap gap-6">
          <span className="rounded-lg bg-accent px-7 py-4 text-15 font-semibold text-bg shadow-accent-btn">
            shadow-accent-btn
          </span>
          <span className="rounded-xl border border-border bg-surface px-7 py-4 text-15 text-text-3 inset-shadow-accent">
            inset-shadow-accent
          </span>
          <span className="rounded-xl border border-border-danger bg-danger-surface px-7 py-4 text-15 text-danger-text inset-shadow-danger">
            inset-shadow-danger
          </span>
        </div>
      </Section>
    </main>
  );
}
