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
  | { kind: "bold"; value: string }
  | { kind: "italic"; value: string }
  | { kind: "math"; value: string };

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
 * Формулы выделяются первыми и дальше не трогаются: в `$a*b$` звёздочка —
 * это умножение, а не курсив, и разобрать её как разметку значило бы сломать
 * формулу.
 */
export function parseInline(text: string): Inline[] {
  const parts: Inline[] = [];
  let rest = text;

  while (rest.length > 0) {
    const start = rest.indexOf("$");
    if (start === -1) break;

    const end = rest.indexOf("$", start + 1);
    if (end === -1) break;

    parts.push(...parseEmphasis(rest.slice(0, start)));
    parts.push({ kind: "math", value: rest.slice(start + 1, end) });
    rest = rest.slice(end + 1);
  }

  parts.push(...parseEmphasis(rest));
  return parts.filter((part) => part.kind !== "text" || part.value.length > 0);
}

/** Жирный и курсив в куске текста, где формул заведомо нет. */
function parseEmphasis(text: string): Inline[] {
  const parts: Inline[] = [];
  const pattern = /\*\*(.+?)\*\*|\*(.+?)\*/g;
  let last = 0;

  for (const match of text.matchAll(pattern)) {
    const at = match.index;
    if (at > last) parts.push({ kind: "text", value: text.slice(last, at) });

    parts.push(
      match[1] !== undefined
        ? { kind: "bold", value: match[1] }
        : { kind: "italic", value: match[2] },
    );
    last = at + match[0].length;
  }

  if (last < text.length) parts.push({ kind: "text", value: text.slice(last) });
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
