import "@testing-library/jest-dom/vitest";

import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// React Testing Library does not auto-clean when `globals` is on in some
// configurations; doing it explicitly keeps tests independent.
afterEach(() => {
  cleanup();
});
