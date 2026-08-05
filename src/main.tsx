import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router";

import { router } from "@/routes/router";
import "@/styles/index.css";

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element #root is missing from index.html");
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
