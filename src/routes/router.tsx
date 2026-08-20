import { createHashRouter, type RouteObject } from "react-router";

import { Home } from "@/routes/Home";

/**
 * Служебные страницы для сверки с макетом. `import.meta.env.DEV` подставляется
 * сборщиком литералом, поэтому в релизе ветка мертва и динамический импорт
 * выбрасывается вместе с самим модулем.
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
    element: <Home />,
  },
  ...devRoutes,
]);
