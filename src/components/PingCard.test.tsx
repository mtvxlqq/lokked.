import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PingCard } from "@/components/PingCard";

// `@tauri-apps/api/core` only works inside a Tauri webview, so the IPC bridge
// is mocked here. The real round-trip is exercised by running `npm run tauri
// dev` — this test covers the wiring on the TypeScript side.
const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("PingCard", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue("pong");
  });

  it("renders the reply from the ping command", async () => {
    render(<PingCard />);

    await expect(screen.findByTestId("ping-reply")).resolves.toHaveTextContent(
      "pong",
    );
    expect(invoke).toHaveBeenCalledWith("ping");
  });
});
