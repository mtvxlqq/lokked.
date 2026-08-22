import { createHashRouter, type RouteObject } from "react-router";

import { AppShell } from "@/components/AppShell";
import { DesktopEvents } from "@/components/DesktopEvents";
import { Settings } from "@/routes/Settings";
import { Streak } from "@/routes/Streak";
import { Timer } from "@/routes/Timer";
import { Timers } from "@/routes/Timers";
import { Zen } from "@/routes/Zen";

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
/**
 * Hash routing, not browser routing: a production Tauri build serves the
 * frontend over the asset protocol, which has no SPA fallback, so reloading on
 * a non-root path would 404. Hash routes keep every path inside `index.html`.
 *
 * Everything hangs off `DesktopEvents`: the global hotkey and the machine
 * waking up can arrive on any screen, so the listener has to sit above all
 * of them — including the black screen, which lives outside `AppShell`.
 */
export const router = createHashRouter([
  {
    element: <DesktopEvents />,
    children: [
      {
        path: "/",
        element: <AppShell />,
        children: [
          { index: true, element: <Timers /> },
          { path: "timer/:subjectId", element: <Timer /> },
          {
            // Отдельным куском: KaTeX со своими шрифтами весит больше всего
            // остального приложения, а нужен только здесь. До экрана карточек
            // ещё надо дойти, а стартовать приложение должно быстро.
            path: "cards",
            lazy: async () => {
              const { Cards } = await import("@/routes/Cards");
              return { Component: Cards };
            },
          },
          { path: "streak", element: <Streak /> },
          {
            // Тоже отдельным куском: на вкладке «Карточка» разбор рисуется
            // той же разметкой, что и на экране карточек, а значит тянет
            // KaTeX.
            path: "stats",
            lazy: async () => {
              const { Stats } = await import("@/routes/Stats");
              return { Component: Stats };
            },
          },
          { path: "settings", element: <Settings /> },
        ],
      },
      // Чёрный экран живёт вне `AppShell`: ему нужен весь экран без навигации.
      { path: "zen", element: <Zen /> },
      {
        // Прогон по колоде — тоже без навигации: на экране должна быть
        // карточка и ничего кроме. KaTeX здесь тот же, что и на экране
        // карточек, поэтому кусок тоже отдельный.
        path: "study/:deckId",
        lazy: async () => {
          const { Study } = await import("@/routes/Study");
          return { Component: Study };
        },
      },
      ...devRoutes,
    ],
  },
]);
