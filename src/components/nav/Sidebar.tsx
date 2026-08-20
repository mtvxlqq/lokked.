import { NavLink } from "react-router";

import { Logo } from "@/components/nav/Logo";
import { NAV_ITEMS } from "@/components/nav/items";
import { cn } from "@/lib/cn";

/**
 * Боковая навигация — только от 768px. Ниже её заменяет `TabBar`: узкий экран
 * целиком уходит под контент.
 */
export function Sidebar() {
  return (
    <aside className="hidden w-62 shrink-0 flex-col gap-8.5 border-r border-hairline bg-surface-sunken px-5.5 py-8.5 md:flex">
      <Logo className="pl-3.5 text-24" />

      <nav aria-label="Разделы" className="flex flex-col gap-1">
        {NAV_ITEMS.map(({ to, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              cn(
                "flex min-h-11 items-center rounded-md px-3.5 py-2.75 text-14.5 transition-colors duration-150 ease-standard",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                isActive
                  ? "bg-raised font-medium text-text"
                  : "text-text-muted-2 hover:text-text-3",
              )
            }
          >
            {label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
