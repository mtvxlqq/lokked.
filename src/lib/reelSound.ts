/**
 * Звук барабана: мягкие щелчки строк, проезжающих мимо.
 *
 * Осцилляторы, а не файл, — по той же причине, что и в [`chime`]: пара
 * десятков коротких «ток» не стоят ни ассета в сборке, ни зависимости от
 * кодека. Громкость низкая, тон низкий, атака не мгновенная: барабан должен
 * слышаться крутящимся, а не трещащим.
 *
 * Щелчки не «играются по таймеру», а расписываются в звуковом времени сразу
 * на весь прокрут: планировщик WebAudio точнее любого `setTimeout`, а
 * барабан крутится ровно 1.3 секунды и не меняет решения по дороге.
 */

/**
 * Кривая замедления ленты — та же, что у `--ease-reel` в `tokens.css`.
 *
 * Дублирование намеренное и единственное: звук обязан замедляться вместе с
 * картинкой, а прочитать значение CSS-переменной здесь неоткуда — щелчки
 * расписываются до того, как браузер начнёт анимацию.
 */
const EASE = { x1: 0.08, y1: 0.82, x2: 0.17, y2: 1 } as const;

/** Ближе этого два щелчка сливаются в треск. */
const MIN_GAP_MS = 30;

/** Самый громкий щелчок — тише, чем сигнал смены фазы. */
const PEAK = 0.035;

/** Последний щелчок — тот, на котором барабан встал. */
const LAST_PEAK = 0.055;

/**
 * Когда щёлкать, в миллисекундах от начала прокрута.
 *
 * Строка щёлкает, пересекая центр окна, поэтому моменты — это времена, в
 * которые лента проехала 1, 2, … `rows` строк. Лента идёт по кривой
 * замедления, значит и щелчки сами собой расходятся к концу.
 *
 * Слишком частые щелчки в начале выбрасываются: ухо всё равно услышит их
 * как один шум, а барабан должен звучать разборчиво.
 */
export function tickTimes(
  rows: number,
  durationMs: number,
  minGapMs: number = MIN_GAP_MS,
): number[] {
  const times: number[] = [];
  let previous = -minGapMs;

  for (let row = 1; row <= rows; row += 1) {
    const at = durationMs * timeAtProgress(row / rows);

    // Последний щелчок не выбрасываем никогда: это и есть остановка.
    if (at - previous < minGapMs && row < rows) continue;

    times.push(at);
    previous = at;
  }

  return times;
}

/**
 * Доля времени, за которую лента проходит долю пути `progress`.
 *
 * Обратная задача к кубической кривой Безье: у CSS вход — время, а нам нужно
 * время по пройденному пути. Кривая монотонна, поэтому хватает деления
 * отрезка пополам; двадцати шагов достаточно для точности в доли
 * миллисекунды.
 */
function timeAtProgress(progress: number): number {
  let low = 0;
  let high = 1;

  for (let step = 0; step < 20; step += 1) {
    const middle = (low + high) / 2;
    if (bezier(middle, EASE.y1, EASE.y2) < progress) low = middle;
    else high = middle;
  }

  return bezier((low + high) / 2, EASE.x1, EASE.x2);
}

/** Координата кубической кривой Безье из (0,0) в (1,1) при параметре `u`. */
function bezier(u: number, a: number, b: number): number {
  const rest = 1 - u;
  return 3 * rest * rest * u * a + 3 * rest * u * u * b + u * u * u;
}

/**
 * Играет прокрут: `rows` строк за `durationMs`.
 *
 * Возвращает функцию, обрывающую звук, — барабан можно покинуть посреди
 * прокрута, и доигрывать щелчки на списке колод было бы странно.
 *
 * Молчит, если система просит меньше движения: там лента не едет, а сразу
 * встаёт, и озвучивать нечего.
 */
export function playSpin(rows: number, durationMs: number): () => void {
  try {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return () => {};
    }

    const context = new AudioContext();
    // Браузер вправе открыть контекст «приостановленным», пока на странице
    // не было ни одного действия. До барабана доходят кликом, так что
    // разрешение обычно уже есть, — а если нет, отказ нас не касается.
    void context.resume().catch(() => {});

    // Общий фильтр: срезает призвук атаки, из-за которого щелчок звучит
    // цифровым.
    const filter = context.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 1200;
    filter.Q.value = 0.7;
    filter.connect(context.destination);

    const times = tickTimes(rows, durationMs);
    for (const [index, at] of times.entries()) {
      const last = index === times.length - 1;
      tick(context, filter, context.currentTime + at / 1000, last);
    }

    // Контекст живёт ровно до конца последнего щелчка: держать открытым
    // звуковое устройство между карточками незачем.
    const tail = (times[times.length - 1] ?? 0) + 400;
    const id = window.setTimeout(() => void context.close(), tail);

    return () => {
      window.clearTimeout(id);
      void context.close();
    };
  } catch {
    // Звук — украшение барабана, а не его работа.
    return () => {};
  }
}

/** Один щелчок: короткое низкое «ток» с плавной атакой и затуханием. */
function tick(
  context: AudioContext,
  destination: AudioNode,
  at: number,
  last: boolean,
): void {
  const oscillator = context.createOscillator();
  const gain = context.createGain();

  oscillator.type = "sine";
  // Тон падает к концу прокрута: так барабан слышится тяжелеющим.
  oscillator.frequency.setValueAtTime(last ? 170 : 240, at);

  const peak = last ? LAST_PEAK : PEAK;
  const decay = last ? 0.22 : 0.09;

  // Не прямоугольник: мгновенная атака — это щелчок динамика, а не барабана.
  gain.gain.setValueAtTime(0.0001, at);
  gain.gain.exponentialRampToValueAtTime(peak, at + 0.008);
  gain.gain.exponentialRampToValueAtTime(0.0001, at + decay);

  oscillator.connect(gain);
  gain.connect(destination);
  oscillator.start(at);
  oscillator.stop(at + decay + 0.02);
}
