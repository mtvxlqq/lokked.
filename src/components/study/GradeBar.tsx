import { Button } from "@/components/ui";
import { cn } from "@/lib/cn";
import type { Grade } from "@/lib/tauri";

/**
 * Четыре оценки в порядке уверенности. «Знаю» — та, что нажимается чаще
 * всех: она выделена рамкой и цветом текста, но не заливкой — акцентная
 * кнопка среди четырёх одинаковых по смыслу тянет руку к себе сильнее, чем
 * того заслуживает оценка.
 */
const GRADES: {
  grade: Grade;
  label: string;
  key: string;
  main?: boolean;
}[] = [
  { grade: "again", label: "Не помню", key: "1" },
  { grade: "hard", label: "С трудом", key: "2" },
  { grade: "good", label: "Знаю", key: "3", main: true },
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
            disabled={disabled}
            onClick={() => onGrade(option.grade)}
            className={cn(
              "flex-col gap-0.5 py-3",
              option.main && "border-border-strong text-text-1",
            )}
          >
            <span>{option.label}</span>
            <span className="text-11 text-text-faint">{option.key}</span>
          </Button>
        ))}
      </div>
    </div>
  );
}
