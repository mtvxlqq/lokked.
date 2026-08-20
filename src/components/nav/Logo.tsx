import { cn } from "@/lib/cn";

/**
 * Логотип. Точка всегда акцентная — это единственная цветная деталь в шапке.
 *
 * Отрицательный правый отступ компенсирует трекинг последней буквы: без него
 * слово выглядит сдвинутым влево относительно оптического центра.
 */
export function Logo({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "-mr-0.5 font-mono font-semibold tracking-timer text-text",
        className,
      )}
    >
      lokked<span className="text-accent">.</span>
    </span>
  );
}
