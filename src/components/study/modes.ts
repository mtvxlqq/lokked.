import type { StudyMode } from "@/lib/tauri";

/**
 * Названия режимов — одни и те же в кнопках колоды и в заголовке прогона.
 * Порядок от привычного к особому: пачка карточек, она же на время, вся
 * колода, только то, что не даётся, и барабан — та же пачка, но карточка не
 * показывается, а выпадает.
 */
export const MODE_NAMES: Record<StudyMode, string> = {
  classic: "Классика",
  blitz: "Блиц",
  marathon: "Марафон",
  weak: "Слабые",
  reel: "Барабан",
};

export const STUDY_MODES: StudyMode[] = [
  "classic",
  "blitz",
  "marathon",
  "weak",
  "reel",
];
