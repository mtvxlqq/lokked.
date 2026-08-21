import { useId, type ReactNode, type TextareaHTMLAttributes } from "react";

import { Field } from "@/components/ui/Field";
import { cn } from "@/lib/cn";

type TextareaProps = Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, "id"> & {
  label: string;
  hint?: ReactNode;
  invalid?: boolean;
  wrapperClassName?: string;
};

/**
 * Многострочное поле: текст карточки, разделители, вставка импорта.
 *
 * Моноширинный шрифт не ставится: сторона карточки — это проза с формулами,
 * а не код, и читать её удобнее тем же шрифтом, каким она потом покажется.
 */
export function Textarea({
  label,
  hint,
  invalid = false,
  wrapperClassName,
  className,
  ...props
}: TextareaProps) {
  const id = useId();

  return (
    <Field
      htmlFor={id}
      label={label}
      hint={hint}
      invalid={invalid}
      className={wrapperClassName}
    >
      <textarea
        id={id}
        aria-invalid={invalid || undefined}
        className={cn(
          "w-full resize-y bg-transparent text-15.5 leading-text text-text-1 outline-none",
          "placeholder:text-text-faint",
          className,
        )}
        {...props}
      />
    </Field>
  );
}
