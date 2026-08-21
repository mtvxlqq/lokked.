import katex from "katex";
import { useMemo, type ReactNode } from "react";

import { cn } from "@/lib/cn";
import { parseMarkdown, type Inline } from "@/lib/markdown";

/**
 * Формула.
 *
 * Единственное место во всём приложении, где в документ попадает готовый
 * HTML, — и он не из карточки, а из KaTeX: текст формулы KaTeX экранирует
 * сам, а `trust: false` (по умолчанию) запрещает командам вроде `\href`
 * и `\htmlClass` что-либо вставлять. Разметка карточки в HTML не
 * превращается никогда — её рисует React.
 *
 * Формула, которую KaTeX не разобрал, показывается исходным текстом:
 * потерять содержимое карточки из-за одной опечатки в `\frac` нельзя.
 */
function Math({ value, display }: { value: string; display: boolean }) {
  const html = useMemo(() => {
    try {
      return katex.renderToString(value, {
        displayMode: display,
        throwOnError: true,
        trust: false,
      });
    } catch {
      return null;
    }
  }, [value, display]);

  if (html === null) {
    return (
      <code
        title="Формула не разобралась — показан исходный текст"
        className="rounded-sm bg-raised px-1 font-mono text-13 text-danger-text"
      >
        {display ? `$$${value}$$` : `$${value}$`}
      </code>
    );
  }

  return (
    <span
      className={cn(display && "block overflow-x-auto py-1 text-center")}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function renderInline(parts: Inline[]): ReactNode {
  return parts.map((part, index) => {
    switch (part.kind) {
      case "text":
        return <span key={index}>{part.value}</span>;
      case "bold":
        return (
          <strong key={index} className="font-semibold text-text-1">
            {part.value}
          </strong>
        );
      case "italic":
        return (
          <em key={index} className="italic">
            {part.value}
          </em>
        );
      case "math":
        return <Math key={index} value={part.value} display={false} />;
    }
  });
}

/**
 * Текст карточки: абзацы, списки и формулы.
 *
 * Формулы и разметку разбирает `@/lib/markdown`, здесь только отрисовка.
 */
export function CardText({
  text,
  className,
}: {
  text: string;
  className?: string;
}) {
  const blocks = useMemo(() => parseMarkdown(text), [text]);

  return (
    <div className={cn("flex flex-col gap-3 text-15 leading-text", className)}>
      {blocks.map((block, index) => {
        switch (block.kind) {
          case "paragraph":
            return <p key={index}>{renderInline(block.inline)}</p>;
          case "math":
            return <Math key={index} value={block.value} display />;
          case "list": {
            const List = block.ordered ? "ol" : "ul";
            return (
              <List
                key={index}
                className={cn(
                  "flex list-outside flex-col gap-1.5 pl-5",
                  block.ordered ? "list-decimal" : "list-disc",
                )}
              >
                {block.items.map((item, itemIndex) => (
                  <li key={itemIndex}>{renderInline(item)}</li>
                ))}
              </List>
            );
          }
        }
      })}
    </div>
  );
}
