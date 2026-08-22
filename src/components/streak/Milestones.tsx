import { formatDay, plural } from "@/lib/format";
import type { StreakMilestone } from "@/lib/tauri";

/**
 * Ближайшие вехи: 7, 30 и 100 дней подряд, с полосой прогресса.
 *
 * Взятая веха не исчезает, а показывает день, когда была взята: серия — это
 * история, и вычёркивать из неё сделанное незачем.
 */
export function Milestones({
  milestones,
  current,
}: {
  milestones: StreakMilestone[];
  current: number;
}) {
  return (
    <ul className="flex flex-col gap-4">
      {milestones.map((milestone) => {
        const progress = Math.min(100, (current / milestone.target) * 100);

        return (
          <li key={milestone.target} className="flex flex-col gap-2">
            <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
              <span className="text-14.5 text-text-1">
                {milestone.target}{" "}
                {plural(milestone.target, ["день", "дня", "дней"])}
              </span>
              <span className="text-12.5 text-text-dim">
                {milestone.reached
                  ? milestone.reached_on
                    ? `взято ${formatDay(milestone.reached_on)}`
                    : "взято"
                  : `осталось ${milestone.remaining} ${plural(milestone.remaining, ["день", "дня", "дней"])}`}
              </span>
            </div>
            <div
              role="progressbar"
              aria-label={`${milestone.target} дней подряд`}
              aria-valuemin={0}
              aria-valuemax={milestone.target}
              aria-valuenow={Math.min(current, milestone.target)}
              className="h-1.5 overflow-hidden rounded-full bg-raised"
            >
              <span
                className="block h-full rounded-full bg-accent"
                style={{ width: `${progress}%` }}
              />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
