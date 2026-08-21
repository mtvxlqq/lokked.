/**
 * Предметы: список, создание, правка, удаление.
 */

import { invoke } from "@tauri-apps/api/core";

export type Subject = {
  id: string;
  name: string;
  /** Слаг палитры (`subject-1` … `subject-8`), не hex. */
  color: string | null;
  icon: string | null;
  position: number;
};

export type SubjectInput = {
  name: string;
  /** `null` при создании — цвет выберет бэкенд. */
  color: string | null;
  icon: string | null;
};

export function listSubjects(): Promise<Subject[]> {
  return invoke<Subject[]>("list_subjects");
}

export function createSubject(input: SubjectInput): Promise<Subject> {
  return invoke<Subject>("create_subject", { input });
}

export function updateSubject(
  id: string,
  input: SubjectInput,
): Promise<Subject> {
  return invoke<Subject>("update_subject", { id, input });
}

export function deleteSubject(id: string): Promise<void> {
  return invoke<void>("delete_subject", { id });
}
