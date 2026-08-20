import { cn } from "@/lib/cn";

type SwitchProps = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** Видимая подпись слева от тумблера; она же — доступное имя. */
  label: string;
  disabled?: boolean;
  className?: string;
};

/**
 * Тумблер в строке настройки: подпись слева, переключатель справа.
 *
 * Нажимается вся строка, а не только сама плашка 44×26 — на телефоне попасть
 * в неё пальцем иначе тяжело.
 */
export function Switch({
  checked,
  onChange,
  label,
  disabled = false,
  className,
}: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        "flex min-h-11 w-full items-center justify-between gap-4 text-left",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        disabled ? "cursor-not-allowed text-text-disabled" : "text-text-1",
        className,
      )}
    >
      <span className="text-14.5">{label}</span>
      <span
        aria-hidden="true"
        className={cn(
          "flex h-6.5 w-11 shrink-0 items-center rounded-full p-0.75 transition-colors duration-150 ease-standard",
          checked
            ? "justify-end bg-accent"
            : "justify-start border border-border-strong bg-raised",
        )}
      >
        <span
          className={cn(
            "size-5 rounded-full",
            checked ? "bg-bg" : "bg-text-dim-2",
          )}
        />
      </span>
    </button>
  );
}
