import { useId, type ReactNode, type SelectHTMLAttributes } from "react";

import { Field } from "@/components/ui/Field";
import { cn } from "@/lib/cn";

type SelectProps = Omit<SelectHTMLAttributes<HTMLSelectElement>, "id"> & {
  label: string;
  hint?: ReactNode;
  invalid?: boolean;
  wrapperClassName?: string;
  children: ReactNode;
};

/**
 * Выпадающий список поверх нативного `<select>`: список опций рисует система,
 * и на мобилке это правильно — там свой барабан выбора, повторять его вручную
 * незачем. Тёмный список получается за счёт `color-scheme: dark` на `<html>`.
 */
export function Select({
  label,
  hint,
  invalid = false,
  wrapperClassName,
  className,
  children,
  ...props
}: SelectProps) {
  const id = useId();

  return (
    <Field
      htmlFor={id}
      label={label}
      hint={hint}
      invalid={invalid}
      className={wrapperClassName}
    >
      <div className="relative flex items-center">
        <select
          id={id}
          aria-invalid={invalid || undefined}
          className={cn(
            "w-full appearance-none bg-transparent pr-6 text-15.5 text-text-1 outline-none",
            className,
          )}
          {...props}
        >
          {children}
        </select>
        <svg
          aria-hidden="true"
          viewBox="0 0 24 24"
          className="pointer-events-none absolute right-0 size-4 stroke-text-dim-2"
          fill="none"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="m6 9.5 6 6 6-6" />
        </svg>
      </div>
    </Field>
  );
}
