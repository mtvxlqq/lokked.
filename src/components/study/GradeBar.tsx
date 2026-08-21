import { Button } from "@/components/ui";
import type { Grade } from "@/lib/tauri";

/**
 * Четыре оценки в порядке уверенности. «Знаю» — главная: это ответ, который
 * даётся чаще всех, и попадать в него надо не глядя.
 */
const GRADES: {
  grade: Grade;
  label: string;
  key: string;
  primary?: boolean;
}[] = [
  { grade: "again", label: "Не помню", key: "1" },
  { grade: "hard", label: "С трудом", key: "2" },
  { grade: "good", label: "Знаю", key: "3", primary: true },
  { grade: "easy", label: "Легко", key: "4" },
];

type GradeBarProps = {
  onGrade: (grade: Grade) => void;
  disabled: boolean;
};

/**
 * Оценки под раскрытой карточкой.
 *
 * У каждой подписана цифра: с клавиатуры прогон идёт быстрее, а искать
 * соответствие «вторая слева — это двойка» глазами не приходится.
 */
export function GradeBar({ onGrade, disabled }: GradeBarProps) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-center text-11 tracking-label text-text-faint uppercase">
        Насколько уверенно ты ответил?
      </p>

      <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
        {GRADES.map((option) => (
          <Button
            key={option.grade}
            variant={option.primary ? "primary" : "secondary"}
            disabled={disabled}
            onClick={() => onGrade(option.grade)}
            className="flex-col gap-0.5 py-3"
          >
            <span>{option.label}</span>
            <span className="text-11 text-text-faint">{option.key}</span>
          </Button>
        ))}
      </div>
    </div>
  );
}
