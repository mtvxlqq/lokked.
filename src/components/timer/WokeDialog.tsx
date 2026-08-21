import { Button, Dialog } from "@/components/ui";
import { formatDuration } from "@/lib/format";

type WokeDialogProps = {
  /** Сколько машина спала. */
  asleepSeconds: number;
  onResume: () => void;
  onKeepPaused: () => void;
};

/**
 * Машина просыпается посреди сессии.
 *
 * Время сна не засчитано и выбора об этом нет: закрытый ноутбук — это не
 * «отвлёкся на минуту», а выключенная машина. Спросить остаётся только одно
 * — продолжаем ли.
 */
export function WokeDialog({
  asleepSeconds,
  onResume,
  onKeepPaused,
}: WokeDialogProps) {
  return (
    <Dialog
      open
      onClose={onKeepPaused}
      title="С возвращением"
      description={`Компьютер спал ${formatDuration(asleepSeconds)}. Таймер стоит на паузе — это время не засчитано.`}
      footer={
        <>
          <Button variant="secondary" onClick={onKeepPaused}>
            Оставить на паузе
          </Button>
          <Button variant="primary" onClick={onResume}>
            Продолжить
          </Button>
        </>
      }
    />
  );
}
