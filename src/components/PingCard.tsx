import { useEffect, useState } from "react";

import { Card } from "@/components/ui/Card";
import { ping } from "@/lib/tauri";

type PingState =
  | { status: "loading" }
  | { status: "ok"; reply: string }
  | { status: "error"; message: string };

/**
 * Calls the Rust `ping` command on mount and renders the reply.
 *
 * This exists purely to prove the Rust ↔ TypeScript bridge works; it is the
 * first thing to check when the window comes up blank.
 */
export function PingCard() {
  const [state, setState] = useState<PingState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;

    ping()
      .then((reply) => {
        if (!cancelled) setState({ status: "ok", reply });
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <Card title="Rust bridge">
      {state.status === "loading" && (
        <p className="text-14 text-text-dim">Calling ping()…</p>
      )}

      {state.status === "ok" && (
        <p className="text-14 text-text-3">
          ping() →{" "}
          <span className="font-mono text-accent-text" data-testid="ping-reply">
            {state.reply}
          </span>
        </p>
      )}

      {state.status === "error" && (
        <p className="text-14 text-danger" role="alert">
          ping() failed: {state.message}
        </p>
      )}
    </Card>
  );
}
