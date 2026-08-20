import { NavLink } from "react-router";

import { NAV_ITEMS } from "@/components/nav/items";
import { cn } from "@/lib/cn";

/**
 * Нижний таб-бар — до 768px. Каждая вкладка занимает равную долю ширины и не
 * ниже 44px: это основная навигация на телефоне, промахиваться по ней нельзя.
 */
export function TabBar() {
  return (
    <nav
      aria-label="Разделы"
      className="flex shrink-0 items-stretch justify-between border-t border-hairline bg-surface-sunken px-3.5 pt-2.5 pb-5.5 md:hidden"
    >
      {NAV_ITEMS.map(({ to, label, Icon }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          className={({ isActive }) =>
            cn(
              "flex min-h-11 flex-1 flex-col items-center justify-center gap-1.25 rounded-md py-0.5",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
              isActive ? "text-accent" : "text-text-faint",
            )
          }
        >
          {({ isActive }) => (
            <>
              <Icon className={cn(isActive && "glow-nav-icon")} />
              <span
                className={cn(
                  "text-10.5",
                  isActive ? "font-medium text-text-2" : "text-text-dim-2",
                )}
              >
                {label}
              </span>
            </>
          )}
        </NavLink>
      ))}
    </nav>
  );
}
