import { describe, expect, it } from "vitest";

import { parseInline, parseMarkdown } from "@/lib/markdown";

describe("parseInline", () => {
  it("выделяет формулу в строке", () => {
    expect(parseInline("Пусть $F'(x)=f(x)$ всюду")).toEqual([
      { kind: "text", value: "Пусть " },
      { kind: "math", value: "F'(x)=f(x)" },
      { kind: "text", value: " всюду" },
    ]);
  });

  it("не трогает звёздочки внутри формулы", () => {
    // Иначе умножение превратилось бы в курсив, а формула — в мусор.
    expect(parseInline("$a*b*c$")).toEqual([{ kind: "math", value: "a*b*c" }]);
  });

  it("разбирает жирный и курсив", () => {
    expect(parseInline("**первообразная** и *интеграл*")).toEqual([
      { kind: "bold", value: "первообразная" },
      { kind: "text", value: " и " },
      { kind: "italic", value: "интеграл" },
    ]);
  });

  it("одинокий доллар остаётся текстом", () => {
    expect(parseInline("стоит 5$ и всё")).toEqual([
      { kind: "text", value: "стоит 5$ и всё" },
    ]);
  });
});

describe("parseMarkdown", () => {
  it("выключная формула — отдельный блок", () => {
    const blocks = parseMarkdown("Получаем\n$$F'(x)=f(x),$$\nоткуда следует");

    expect(blocks).toEqual([
      { kind: "paragraph", inline: [{ kind: "text", value: "Получаем" }] },
      { kind: "math", value: "F'(x)=f(x)," },
      {
        kind: "paragraph",
        inline: [{ kind: "text", value: "откуда следует" }],
      },
    ]);
  });

  it("соседние строки — один абзац, пустая строка — новый", () => {
    const blocks = parseMarkdown("Первая строка\nВторая строка\n\nНовый абзац");

    expect(blocks).toHaveLength(2);
    expect(blocks[0]).toEqual({
      kind: "paragraph",
      inline: [{ kind: "text", value: "Первая строка Вторая строка" }],
    });
  });

  it("собирает нумерованный список", () => {
    const blocks = parseMarkdown("Условия:\n1. первое\n2. второе");

    expect(blocks[1]).toEqual({
      kind: "list",
      ordered: true,
      items: [
        [{ kind: "text", value: "первое" }],
        [{ kind: "text", value: "второе" }],
      ],
    });
  });

  it("маркированный список отделён от нумерованного", () => {
    const blocks = parseMarkdown("- один\n- два\n1. первое");

    expect(blocks).toHaveLength(2);
    expect(blocks[0]).toMatchObject({ kind: "list", ordered: false });
    expect(blocks[1]).toMatchObject({ kind: "list", ordered: true });
  });

  it("формулы внутри пункта списка сохраняются", () => {
    const blocks = parseMarkdown("- при $x>0$ верно");

    expect(blocks[0]).toEqual({
      kind: "list",
      ordered: false,
      items: [
        [
          { kind: "text", value: "при " },
          { kind: "math", value: "x>0" },
          { kind: "text", value: " верно" },
        ],
      ],
    });
  });

  it("переносы Windows не меняют разбор", () => {
    expect(parseMarkdown("Строка\r\n\r\nВторая")).toHaveLength(2);
  });

  it("пустой текст даёт пустой разбор", () => {
    expect(parseMarkdown("   \n\n  ")).toEqual([]);
  });

  it("многострочная выключная формула остаётся одним блоком", () => {
    const blocks = parseMarkdown("$$\\begin{cases} a \\\\ b \\end{cases}$$");

    expect(blocks).toEqual([
      { kind: "math", value: "\\begin{cases} a \\\\ b \\end{cases}" },
    ]);
  });
});
