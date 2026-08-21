import { Button, Dialog } from "@/components/ui";
import { formatDuration } from "@/lib/format";

type AwayDialogProps = {
  /** Сколько времени приложение было не на экране. */
  awaySeconds: number;
  onKeep: () => void;
  onDiscard: () => void;
};

/**
 * Что делать со временем, пока приложения не было видно.
 *
 * Показывается только после действительно долгого отсутствия — порог решает
 * бэкенд (`core::session::AWAY_PROMPT_SECONDS`), а не этот компонент.
 */
export function AwayDialog({
  awaySeconds,
  onKeep,
  onDiscard,
}: AwayDialogProps) {
  return (
    <Dialog
      open
      onClose={onKeep}
      title="Тебя не было"
      description={`Приложение было свёрнуто ${formatDuration(awaySeconds)}. Засчитать это время как учёбу?`}
      footer={
        <>
          <Button variant="secondary" onClick={onDiscard}>
            Отбросить
          </Button>
          <Button variant="primary" onClick={onKeep}>
            Засчитать
          </Button>
        </>
      }
    />
  );
}
