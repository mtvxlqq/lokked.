import { createHashRouter } from "react-router";

import { Home } from "@/routes/Home";

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
]);
