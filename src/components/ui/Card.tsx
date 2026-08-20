import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/cn";

type CardProps = HTMLAttributes<HTMLDivElement> & {
  /** Заголовок панели; слева, тем же кеглем, что и текст строки списка. */
  title?: ReactNode;
  /** Приписка справа от заголовка: «граница дня — 04:00». */
  aside?: ReactNode;
  children: ReactNode;
};

/**
 * Карточка-панель: основной контейнер контента на всех экранах.
 *
 * Паддинг разный по ширине экрана — на мобилке 16px, на десктопе 28px,
 * иначе на 380px от контента остаётся полоска.
 */
export function Card({
  title,
  aside,
  className,
  children,
  ...props
}: CardProps) {
  return (
    <div
      className={cn(
        "flex flex-col gap-4.5 rounded-xl border border-border bg-surface p-4 sm:p-7",
        className,
      )}
      {...props}
    >
      {(title || aside) && (
        <div className="flex items-baseline justify-between gap-2.5">
          {title && (
            <span className="text-14.5 font-medium text-text-1">{title}</span>
          )}
          {aside && <span className="text-12.5 text-text-dim-2">{aside}</span>}
        </div>
      )}
      {children}
    </div>
  );
}
