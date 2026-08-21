/**
 * Сообщить о смене фазы: системное уведомление и короткий сигнал.
 *
 * Ни то ни другое не должно ронять экран таймера — если уведомления
 * запрещены, а звук браузер играть отказался, сессия всё равно идёт. Поэтому
 * каждая функция глушит свои ошибки и ничего не обещает вызывающему.
 */
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

/**
 * Короткий сигнал через WebAudio.
 *
 * Осциллятор, а не файл: два тона на полсекунды не стоят ни лишнего ассета
 * в сборке, ни зависимости от кодека. Громкость держим низкой — сигнал
 * говорит «фаза сменилась», а не будит соседей.
 */
export function chime(): void {
  try {
    const context = new AudioContext();
    const oscillator = context.createOscillator();
    const gain = context.createGain();

    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(660, context.currentTime);
    oscillator.frequency.setValueAtTime(880, context.currentTime + 0.12);

    // Затухание, а не резкий обрыв: щелчок в конце ноты слышен и неприятен.
    gain.gain.setValueAtTime(0.0001, context.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.12, context.currentTime + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, context.currentTime + 0.4);

    oscillator.connect(gain);
    gain.connect(context.destination);
    oscillator.start();
    oscillator.stop(context.currentTime + 0.42);
    oscillator.onended = () => void context.close();
  } catch {
    // Звук — приятное дополнение, а не часть работы таймера.
  }
}

/** Системное уведомление; спрашивает разрешение при первой попытке. */
export async function notify(title: string, body: string): Promise<void> {
  try {
    const granted =
      (await isPermissionGranted()) ||
      (await requestPermission()) === "granted";

    if (granted) sendNotification({ title, body });
  } catch {
    // Пользователь мог запретить уведомления — это его право.
  }
}
