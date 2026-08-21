import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { CardText } from "@/components/cards/CardText";

describe("CardText", () => {
  it("рисует формулу, а не её исходник", () => {
    const { container } = render(<CardText text="Пусть $F'(x)=f(x)$ всюду" />);

    // KaTeX оставляет исходник в <annotation> для копирования, поэтому
    // проверяем по разметке, а не по тексту.
    expect(container.querySelector(".katex")).not.toBeNull();
    expect(screen.getByText(/Пусть/)).toBeInTheDocument();
  });

  it("сломанную формулу показывает исходным текстом, а не теряет", () => {
    // В фигурных скобках, а не строкой в JSX: в JSX-строке обратный слэш
    // остался бы двумя символами и до KaTeX доехала бы другая формула.
    const { container } = render(<CardText text={"$\\frac{1}{$"} />);

    expect(container.querySelector(".katex")).toBeNull();
    expect(screen.getByText("$\\frac{1}{$")).toBeInTheDocument();
  });

  it("не превращает текст карточки в разметку", () => {
    // Карточка с тегом внутри — это текст, а не HTML: единственный HTML на
    // экране приходит из KaTeX.
    const { container } = render(
      <CardText text="<img src=x onerror=alert(1)>" />,
    );

    expect(container.querySelector("img")).toBeNull();
    expect(
      screen.getByText("<img src=x onerror=alert(1)>"),
    ).toBeInTheDocument();
  });

  it("выделяет жирный и курсив", () => {
    render(<CardText text="**первообразная** и *интеграл*" />);

    // Текст лежит внутри выделения, а не является им: выделение вложенное,
    // потому что внутри него может быть формула.
    expect(screen.getByText("первообразная").closest("strong")).not.toBeNull();
    expect(screen.getByText("интеграл").closest("em")).not.toBeNull();
  });

  it("собирает список", () => {
    render(<CardText text={"1. первое\n2. второе"} />);

    expect(screen.getAllByRole("listitem")).toHaveLength(2);
  });
});
