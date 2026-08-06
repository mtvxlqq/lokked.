import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Home } from "@/routes/Home";

// `@tauri-apps/api/core` only works inside a Tauri webview, so the IPC bridge
// is mocked here. The real round-trip is exercised by running `npm run tauri
// dev` — this test covers the wiring on the TypeScript side.
const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("Home", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue("pong");
  });

  it("renders the app title and the reply from the ping command", async () => {
    render(<Home />);

    expect(
      screen.getByRole("heading", { level: 1, name: "Lokked" }),
    ).toBeInTheDocument();

    await expect(screen.findByTestId("ping-reply")).resolves.toHaveTextContent(
      "pong",
    );
    expect(invoke).toHaveBeenCalledWith("ping");
  });
});
