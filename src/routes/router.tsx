import { createHashRouter, type RouteObject } from "react-router";

import { AppShell } from "@/components/AppShell";
import { Cards } from "@/routes/Cards";
import { Settings } from "@/routes/Settings";
import { Stats } from "@/routes/Stats";
import { Timers } from "@/routes/Timers";

/**
 * Служебные страницы для сверки с макетом. Живут вне `AppShell`: им нужна вся
 * ширина окна, а навигация только мешает.
 *
 * `import.meta.env.DEV` подставляется сборщиком литералом, поэтому в релизе
 * ветка мертва и динамические импорты выбрасываются вместе с модулями.
 */
const devRoutes: RouteObject[] = import.meta.env.DEV
  ? [
      {
        path: "dev/tokens",
        lazy: async () => {
          const { Tokens } = await import("@/routes/dev/Tokens");
          return { Component: Tokens };
        },
      },
      {
        path: "dev/ui",
        lazy: async () => {
          const { Ui } = await import("@/routes/dev/Ui");
          return { Component: Ui };
        },
      },
    ]
  : [];

/**
 * Hash routing, not browser routing: a production Tauri build serves the
 * frontend over the asset protocol, which has no SPA fallback, so reloading on
 * a non-root path would 404. Hash routes keep every path inside `index.html`.
 */
export const router = createHashRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Timers /> },
      { path: "cards", element: <Cards /> },
      { path: "stats", element: <Stats /> },
      { path: "settings", element: <Settings /> },
    ],
  },
  ...devRoutes,
]);
