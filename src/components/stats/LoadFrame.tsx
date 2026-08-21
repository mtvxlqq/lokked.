import type { ReactNode } from "react";

import { Button, Card } from "@/components/ui";
import type { LoadState } from "@/components/stats/useStatsData";

type LoadFrameProps = {
  state: LoadState;
  error: string | null;
  onRetry: () => void;
  children: ReactNode;
};

/**
 * Одинаковая обёртка для всех вкладок: «Загрузка…», ошибка с повтором или
 * содержимое.
 */
export function LoadFrame({ state, error, onRetry, children }: LoadFrameProps) {
  if (state === "loading") {
    return <p className="text-14 text-text-dim">Загрузка…</p>;
  }

  if (state === "failed") {
    return (
      <Card title="Не удалось загрузить статистику">
        <p className="text-14 text-danger-text" role="alert">
          {error}
        </p>
        <div>
          <Button variant="secondary" onClick={onRetry}>
            Повторить
          </Button>
        </div>
      </Card>
    );
  }

  return <>{children}</>;
}
