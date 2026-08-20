import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/cn";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

const VARIANTS: Record<ButtonVariant, string> = {
  /** Одно главное действие на экран: «Пауза», «Прислать ссылку для входа». */
  primary:
    "bg-accent text-bg font-semibold hover:bg-accent-text disabled:bg-raised disabled:text-text-disabled",
  /** Всё остальное рядом с главным: «Отвлёкся», «Стоп», «Выйти». */
  secondary:
    "border border-border-strong text-text-4 hover:border-border-mute hover:text-text-2 disabled:border-border disabled:text-text-disabled",
  /** Акцентная, но не главная: «Старт» в строке предмета. */
  ghost:
    "border border-border-accent bg-accent-surface text-accent-text hover:border-accent disabled:border-border disabled:bg-transparent disabled:text-text-disabled",
  /** Необратимое: «Удалить аккаунт». */
  danger:
    "border border-border-danger text-danger hover:border-danger disabled:border-border disabled:text-text-disabled",
};

const SIZES: Record<ButtonSize, string> = {
  sm: "px-5 py-2.25 text-13.5",
  md: "px-7 py-3.75 text-15",
  lg: "px-11.5 py-4.25 text-15.5",
};

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Растянуть на всю ширину — так кнопки складываются в колонку на мобилке. */
  block?: boolean;
  children: ReactNode;
};

/**
 * Кнопка приложения.
 *
 * `min-h-11` стоит на всех размерах, включая `sm`: 44×44 — минимальный тач-таргет,
 * и на мобилке он важнее компактности.
 */
export function Button({
  variant = "secondary",
  size = "md",
  block = false,
  className,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        "inline-flex min-h-11 items-center justify-center rounded-lg text-center transition-colors duration-150 ease-standard",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        "disabled:cursor-not-allowed disabled:hover:border-border",
        VARIANTS[variant],
        SIZES[size],
        block && "w-full",
        className,
      )}
      {...props}
    />
  );
}
