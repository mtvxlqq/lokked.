import { useNavigate } from "react-router";

/**
 * Чёрный экран.
 *
 * Пока заглушка: полноэкранный режим, гашение цифр и жесты — это M7.
 * Маршрут заведён сейчас, чтобы кнопка на экране сессии вела в настоящее
 * место, а не в никуда.
 */
export function Zen() {
  const navigate = useNavigate();

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-6 bg-bg-zen px-6 text-center">
      <p className="text-15 text-text-zen-dim">
        Чёрный экран появится позже — пока здесь пусто.
      </p>
      <button
        type="button"
        onClick={() => void navigate(-1)}
        className="min-h-11 text-13 tracking-label text-text-zen-dim-2 uppercase focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        Вернуться к сессии
      </button>
    </div>
  );
}
