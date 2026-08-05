import { useEffect, useState } from "react";

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
    <div className="rounded-xl border border-surface-700 bg-surface-800 px-6 py-5">
      <h2 className="text-sm font-medium tracking-wide text-content-500 uppercase">
        Rust bridge
      </h2>

      {state.status === "loading" && (
        <p className="mt-2 text-content-300">Calling ping()…</p>
      )}

      {state.status === "ok" && (
        <p className="mt-2 text-content-300">
          ping() →{" "}
          <span className="font-mono text-accent-400" data-testid="ping-reply">
            {state.reply}
          </span>
        </p>
      )}

      {state.status === "error" && (
        <p className="mt-2 text-red-400" role="alert">
          ping() failed: {state.message}
        </p>
      )}
    </div>
  );
}
