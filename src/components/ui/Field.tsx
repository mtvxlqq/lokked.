import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

type FieldProps = {
  htmlFor: string;
  label: string;
  /** Пояснение или текст ошибки под полем. */
  hint?: ReactNode;
  invalid?: boolean;
  className?: string;
  children: ReactNode;
};

/**
 * Общая оболочка поля ввода: подпись сверху мелкой капителью, само поле —
 * крупнее и ярче. В макете подпись и значение живут в одной рамке, а не
 * подпись снаружи, — так поля читаются как строки списка.
 *
 * Используется `Input` и `Select`; отдельно не экспортируется.
 */
export function Field({
  htmlFor,
  label,
  hint,
  invalid = false,
  className,
  children,
}: FieldProps) {
  return (
    <div className={cn("flex flex-col gap-1.5", className)}>
      <div
        className={cn(
          "flex flex-col gap-1.25 rounded-lg border bg-surface px-4.5 py-3.5",
          "focus-within:border-border-accent",
          invalid ? "border-border-danger" : "border-border",
        )}
      >
        <label
          htmlFor={htmlFor}
          className="text-11 tracking-label text-text-faint uppercase"
        >
          {label}
        </label>
        {children}
      </div>
      {hint && (
        <span
          className={cn(
            "text-12.5",
            invalid ? "text-danger-text" : "text-text-dim",
          )}
        >
          {hint}
        </span>
      )}
    </div>
  );
}
