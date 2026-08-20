import { useId, type InputHTMLAttributes, type ReactNode } from "react";

import { Field } from "@/components/ui/Field";
import { cn } from "@/lib/cn";

type InputProps = Omit<InputHTMLAttributes<HTMLInputElement>, "id"> & {
  label: string;
  hint?: ReactNode;
  invalid?: boolean;
  /** Классы для внешней обёртки; само поле ширину не задаёт. */
  wrapperClassName?: string;
};

export function Input({
  label,
  hint,
  invalid = false,
  wrapperClassName,
  className,
  ...props
}: InputProps) {
  const id = useId();

  return (
    <Field
      htmlFor={id}
      label={label}
      hint={hint}
      invalid={invalid}
      className={wrapperClassName}
    >
      <input
        id={id}
        aria-invalid={invalid || undefined}
        className={cn(
          "w-full bg-transparent text-15.5 text-text-1 outline-none",
          "placeholder:text-text-faint",
          className,
        )}
        {...props}
      />
    </Field>
  );
}
