import { cn } from "@/lib/cn";

type Option<T extends string> = {
  value: T;
  label: string;
};

type SegmentedControlProps<T extends string> = {
  /** Что выбирают: попадает в `aria-label` группы. */
  label: string;
  value: T;
  options: Option<T>[];
  onChange: (value: T) => void;
  className?: string;
};

/**
 * Переключатель из нескольких кнопок: вкладка, период, режим.
 *
 * Кнопки, а не `<select>`: вариантов мало, все они помещаются на экран, и
 * переключение одним касанием на мобилке важнее компактности. Выбранное
 * состояние помечено `aria-pressed`, а не только цветом.
 *
 * На узком экране кнопки переносятся на вторую строку целиком — сжимать их
 * ниже 44px нельзя.
 */
export function SegmentedControl<T extends string>({
  label,
  value,
  options,
  onChange,
  className,
}: SegmentedControlProps<T>) {
  return (
    <div
      role="group"
      aria-label={label}
      className={cn(
        "flex flex-wrap gap-1 rounded-lg border border-border bg-surface-sunken p-1",
        className,
      )}
    >
      {options.map((option) => {
        const selected = option.value === value;

        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={selected}
            onClick={() => onChange(option.value)}
            className={cn(
              "min-h-11 flex-1 rounded-md px-4 text-13.5 whitespace-nowrap transition-colors duration-150 ease-standard",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
              selected
                ? "bg-raised text-text-1"
                : "text-text-dim hover:text-text-3",
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
