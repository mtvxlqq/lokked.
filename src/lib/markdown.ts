/**
 * Разбор текста карточки: формулы и минимальный markdown.
 *
 * Ровно то подмножество, которым размечены карточки: выключные формулы
 * `$$…$$` отдельной строкой, формулы в строке `$…$`, **жирный**, *курсив* и
 * списки. Ни таблиц, ни ссылок, ни картинок — их в карточках нет, а каждый
 * лишний случай в разборе это ещё один способ показать не то.
 *
 * Разбор чистый: на выходе дерево, которое рисует React, и никакого HTML.
 * Единственный HTML во всём экране — тот, что генерирует KaTeX из формулы;
 * текст карточки в разметку не превращается никогда, поэтому вставить через
 * карточку чужой тег невозможно.
 */

export type Inline =
  | { kind: "text"; value: string }
  | { kind: "math"; value: string }
  // Выделение вложенное, а не плоское: «**непрерывным в точке $x_0 \in X$**»
  // — это жирный, внутри которого формула, и разобрать его как три куска
  // текста значит показать студенту звёздочки.
  | { kind: "bold"; children: Inline[] }
  | { kind: "italic"; children: Inline[] };

export type Block =
  | { kind: "paragraph"; inline: Inline[] }
  | { kind: "math"; value: string }
  | { kind: "list"; ordered: boolean; items: Inline[][] };

/** Строка, целиком занятая выключной формулой. */
const DISPLAY_MATH = /^\$\$(.+)\$\$$/s;
/** Маркер списка: «- », «* » или «1. ». */
const BULLET = /^[-*]\s+(.*)$/;
const ORDERED = /^\d+[.)]\s+(.*)$/;

/**
 * Разбирает строку на формулы, жирный и курсив.
 *
 * Один проход слева направо: что открылось раньше, то и разбирается первым.
 * Поэтому в `$a*b$` звёздочка — умножение (формула началась раньше), а в
 * `**текст $x$**` формула оказывается внутри жирного (жирный начался
 * раньше). Незакрытый маркер остаётся обычным текстом: `5$` — это пять
 * долларов, а не начало формулы.
 */
export function parseInline(text: string): Inline[] {
  const parts: Inline[] = [];
  let plain = "";
  let index = 0;

  function flush() {
    if (plain.length > 0) {
      parts.push({ kind: "text", value: plain });
      plain = "";
    }
  }

  while (index < text.length) {
    if (text[index] === "$") {
      const end = text.indexOf("$", index + 1);
      if (end !== -1) {
        flush();
        parts.push({ kind: "math", value: text.slice(index + 1, end) });
        index = end + 1;
        continue;
      }
    } else if (text.startsWith("**", index)) {
      const end = text.indexOf("**", index + 2);
      if (end !== -1) {
        flush();
        parts.push({
          kind: "bold",
          children: parseInline(text.slice(index + 2, end)),
        });
        index = end + 2;
        continue;
      }
    } else if (text[index] === "*") {
      const end = text.indexOf("*", index + 1);
      if (end !== -1) {
        flush();
        parts.push({
          kind: "italic",
          children: parseInline(text.slice(index + 1, end)),
        });
        index = end + 1;
        continue;
      }
    }

    plain += text[index];
    index += 1;
  }

  flush();
  return parts;
}

/**
 * Разбирает текст карточки на блоки.
 *
 * Подряд идущие обычные строки — один абзац: перенос строки в источнике
 * означает конец предложения, а не конец абзаца. Пустая строка, выключная
 * формула и список абзац закрывают.
 */
export function parseMarkdown(text: string): Block[] {
  const blocks: Block[] = [];
  let paragraph: string[] = [];
  let list: { ordered: boolean; items: string[] } | null = null;

  function closeParagraph() {
    if (paragraph.length === 0) return;
    blocks.push({
      kind: "paragraph",
      inline: parseInline(paragraph.join(" ")),
    });
    paragraph = [];
  }

  function closeList() {
    if (!list) return;
    blocks.push({
      kind: "list",
      ordered: list.ordered,
      items: list.items.map(parseInline),
    });
    list = null;
  }

  for (const raw of text.replace(/\r\n/g, "\n").split("\n")) {
    const line = raw.trim();

    if (line.length === 0) {
      closeParagraph();
      closeList();
      continue;
    }

    const display = DISPLAY_MATH.exec(line);
    if (display) {
      closeParagraph();
      closeList();
      blocks.push({ kind: "math", value: display[1].trim() });
      continue;
    }

    const bullet = BULLET.exec(line);
    const ordered = ORDERED.exec(line);
    if (bullet || ordered) {
      closeParagraph();
      const item = (bullet ?? ordered)![1];
      const isOrdered = ordered !== null && bullet === null;

      // Смена вида списка начинает новый список, а не продолжает старый.
      if (list && list.ordered !== isOrdered) closeList();
      list ??= { ordered: isOrdered, items: [] };
      list.items.push(item);
      continue;
    }

    closeList();
    paragraph.push(line);
  }

  closeParagraph();
  closeList();
  return blocks;
}

/**
 * Текст без разметки: для доступного имени строки списка и подсказки при
 * наведении.
 *
 * Формула превращается в свой исходник без долларов — прочитать вслух
 * `\frac{1}{2}` всё же лучше, чем «доллар слэш фрак».
 */
export function plainText(text: string): string {
  function flatten(parts: Inline[]): string {
    return parts
      .map((part) =>
        part.kind === "bold" || part.kind === "italic"
          ? flatten(part.children)
          : part.value,
      )
      .join("");
  }

  return parseMarkdown(text)
    .flatMap((block) => {
      if (block.kind === "math") return [block.value];
      if (block.kind === "paragraph") return [flatten(block.inline)];
      return block.items.map(flatten);
    })
    .join(" ")
    .replace(/\s+/g, " ")
    .trim();
}
