import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

type EmptyStateProps = {
  title: string;
  description?: ReactNode;
  /** Иконка 24×24 в том же стиле, что и навигация: обводка, без заливки. */
  icon?: ReactNode;
  /** Кнопка призыва к действию. */
  action?: ReactNode;
  className?: string;
};

/**
 * Пустой экран. Не «нет данных», а приглашение: заголовок объясняет, чего тут
 * пока нет, а кнопка сразу даёт это создать.
 */
export function EmptyState({
  title,
  description,
  icon,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed border-border-mute px-6 py-12 text-center",
        className,
      )}
    >
      {icon && <span className="text-text-faint">{icon}</span>}
      <div className="flex max-w-md flex-col gap-2">
        <span className="text-17 font-medium text-text-1">{title}</span>
        {description && (
          <p className="text-14 leading-text text-text-dim">{description}</p>
        )}
      </div>
      {action}
    </div>
  );
}
