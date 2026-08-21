import type { Subject } from "@/lib/tauri";

/**
 * Палитра предметов: слаг из БД → классы Tailwind.
 *
 * Карта, а не шаблонная строка `bg-subject-${n}`: Tailwind собирает классы,
 * сканируя исходники, и склеенное в рантайме имя в сборку просто не попадёт.
 */
export const SUBJECT_COLORS = [
  "subject-1",
  "subject-2",
  "subject-3",
  "subject-4",
  "subject-5",
  "subject-6",
  "subject-7",
  "subject-8",
] as const;

export type SubjectColor = (typeof SUBJECT_COLORS)[number];

const BACKGROUNDS: Record<SubjectColor, string> = {
  "subject-1": "bg-subject-1",
  "subject-2": "bg-subject-2",
  "subject-3": "bg-subject-3",
  "subject-4": "bg-subject-4",
  "subject-5": "bg-subject-5",
  "subject-6": "bg-subject-6",
  "subject-7": "bg-subject-7",
  "subject-8": "bg-subject-8",
};

/**
 * Класс фона для цвета предмета. Предмет без цвета (или с неизвестным слагом
 * из будущей версии) получает приглушённую границу, а не пустое место.
 */
export function subjectBackground(color: Subject["color"]): string {
  if (color && color in BACKGROUNDS) {
    return BACKGROUNDS[color as SubjectColor];
  }
  return "bg-border-strong";
}
