import { useId, type InputHTMLAttributes, type ReactNode } from "react";

import { Field } from "@/components/ui/Field";
import { cn } from "@/lib/cn";

type SliderProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "id" | "type"
> & {
  label: string;
  hint?: ReactNode;
  /** Что стоит справа от дорожки: слово или число, а не «сырое» значение. */
  valueLabel?: ReactNode;
  wrapperClassName?: string;
};

/**
 * Ползунок поверх нативного `<input type="range">`: клавиатура, касание и
 * чтение с экрана достаются даром, остаётся только нарисовать дорожку.
 *
 * Высота — 44px вместе с прозрачными полями вокруг дорожки: попасть пальцем
 * в линию толщиной шесть пикселей иначе невозможно.
 */
export function Slider({
  label,
  hint,
  valueLabel,
  wrapperClassName,
  className,
  ...props
}: SliderProps) {
  const id = useId();

  return (
    <Field htmlFor={id} label={label} hint={hint} className={wrapperClassName}>
      <div className="flex items-center gap-4">
        <input
          id={id}
          type="range"
          className={cn(
            "h-11 w-full cursor-pointer appearance-none bg-transparent outline-none",
            "[&::-webkit-slider-runnable-track]:h-1.5 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-raised",
            "[&::-webkit-slider-thumb]:-mt-2 [&::-webkit-slider-thumb]:size-5.5 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-accent",
            "[&::-moz-range-track]:h-1.5 [&::-moz-range-track]:rounded-full [&::-moz-range-track]:bg-raised",
            "[&::-moz-range-thumb]:size-5.5 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:bg-accent",
            "focus-visible:[&::-webkit-slider-thumb]:outline-2 focus-visible:[&::-webkit-slider-thumb]:outline-offset-2 focus-visible:[&::-webkit-slider-thumb]:outline-accent",
            "focus-visible:[&::-moz-range-thumb]:outline-2 focus-visible:[&::-moz-range-thumb]:outline-offset-2 focus-visible:[&::-moz-range-thumb]:outline-accent",
            className,
          )}
          {...props}
        />
        {valueLabel && (
          <span className="shrink-0 text-15.5 text-text-1">{valueLabel}</span>
        )}
      </div>
    </Field>
  );
}
