import { useEffect, useId, useRef, type ReactNode } from "react";

import { cn } from "@/lib/cn";

type DialogProps = {
  open: boolean;
  onClose: () => void;
  title: string;
  /** Пояснение под заголовком. */
  description?: ReactNode;
  /** Кнопки внизу; на мобилке складываются в колонку. */
  footer?: ReactNode;
  className?: string;
  children?: ReactNode;
};

/**
 * Модальное окно поверх нативного `<dialog>`: Esc, возврат фокуса и запрет
 * взаимодействия с фоном достаются из платформы, а не переписываются руками.
 */
export function Dialog({
  open,
  onClose,
  title,
  description,
  footer,
  className,
  children,
}: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;

    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onCancel={onClose}
      aria-labelledby={titleId}
      className={cn(
        "m-auto w-full max-w-lg rounded-2xl border border-border bg-surface p-6 text-text-2 sm:p-7",
        "backdrop:bg-bg-zen/70",
        className,
      )}
    >
      <div className="flex flex-col gap-4.5">
        <div className="flex flex-col gap-2">
          <h2
            id={titleId}
            className="text-19 font-semibold tracking-tight text-text"
          >
            {title}
          </h2>
          {description && (
            <p className="text-14 leading-text text-text-muted">
              {description}
            </p>
          )}
        </div>

        {children}

        {footer && (
          <div className="flex flex-col gap-2.5 sm:flex-row sm:justify-end">
            {footer}
          </div>
        )}
      </div>
    </dialog>
  );
}
