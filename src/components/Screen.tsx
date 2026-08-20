import type { ReactNode } from "react";

type ScreenProps = {
  title: string;
  /** Действия справа от заголовка; на узком экране переносятся под него. */
  actions?: ReactNode;
  children: ReactNode;
};

/**
 * Общая рамка экрана: заголовок и вертикальный ритм под ним. Отступы от краёв
 * задаёт `AppShell`, здесь только расстояния между блоками.
 */
export function Screen({ title, actions, children }: ScreenProps) {
  return (
    <div className="flex flex-1 flex-col gap-5 sm:gap-8">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <h1 className="text-21 font-semibold tracking-title text-text sm:text-30">
          {title}
        </h1>
        {actions}
      </header>

      {children}
    </div>
  );
}
