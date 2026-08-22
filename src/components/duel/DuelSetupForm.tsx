import { useState } from "react";

import { Button, Card, Input, Select } from "@/components/ui";
import type { Deck } from "@/lib/tauri";

/** Сколько игроков влезает за одно устройство. */
const MIN_PLAYERS = 2;
const MAX_PLAYERS = 4;

/** Сколько карточек разыгрывается. */
const CARD_COUNTS = [5, 10, 15, 20, 30, 40, 50];

/** Сколько секунд даётся на карточку. */
const SECONDS = [10, 15, 20, 30, 45, 60];

export type DuelChoice = {
  deckId: string;
  players: string[];
  cards: number;
  secondsPerCard: number;
};

type DuelSetupFormProps = {
  decks: Deck[];
  /** Колода, которую уже выбрал барабан, если крутили. */
  picked: Deck | null;
  busy: boolean;
  error: string | null;
  onSpin: () => void;
  onStart: (choice: DuelChoice) => void;
  onLeave: () => void;
};

/**
 * Настройка дуэли: кто играет, на чём и сколько.
 *
 * Колоду можно выбрать самому или отдать барабану — как выбор арены перед
 * боем: либо ткнул пальцем, либо «рандом», и пусть решает он.
 */
export function DuelSetupForm({
  decks,
  picked,
  busy,
  error,
  onSpin,
  onStart,
  onLeave,
}: DuelSetupFormProps) {
  const [names, setNames] = useState<string[]>(["Ты", "Артём"]);
  const [deckId, setDeckId] = useState<string>("");
  const [cards, setCards] = useState(20);
  const [seconds, setSeconds] = useState(20);

  const chosen = picked?.id ?? deckId;
  const deck = decks.find((item) => item.id === chosen) ?? null;
  const enough = deck !== null && deck.card_count >= cards;

  function rename(index: number, name: string) {
    setNames(names.map((old, position) => (position === index ? name : old)));
  }

  return (
    <div className="flex flex-col gap-2.5">
      <Card title="Игроки" aside={`${names.length} из ${MAX_PLAYERS}`}>
        <ul className="flex flex-col gap-2.5">
          {names.map((name, index) => (
            <li key={index} className="flex items-end gap-2.5">
              <Input
                label={index === 0 ? "Хозяин устройства" : `Игрок ${index + 1}`}
                hint={
                  index === 0
                    ? "Только твои ответы попадут в личную статистику"
                    : undefined
                }
                value={name}
                wrapperClassName="flex-1"
                onChange={(event) => rename(index, event.target.value)}
              />
              {index >= MIN_PLAYERS && (
                <Button
                  variant="ghost"
                  onClick={() =>
                    setNames(names.filter((_, position) => position !== index))
                  }
                >
                  Убрать
                </Button>
              )}
            </li>
          ))}
        </ul>

        {names.length < MAX_PLAYERS && (
          <div>
            <Button
              variant="secondary"
              onClick={() => setNames([...names, `Игрок ${names.length + 1}`])}
            >
              Добавить игрока
            </Button>
          </div>
        )}
      </Card>

      <Card title="Колода">
        <Select
          label="На чём соревнуемся"
          value={chosen}
          onChange={(event) => setDeckId(event.target.value)}
        >
          <option value="">Выбери колоду</option>
          {decks.map((item) => (
            <option key={item.id} value={item.id}>
              {item.name} · {item.card_count}
            </option>
          ))}
        </Select>

        <div className="flex flex-wrap items-center gap-3">
          <Button variant="secondary" disabled={busy} onClick={onSpin}>
            Крутить барабан
          </Button>
          <span className="text-12.5 text-text-dim">
            {picked
              ? `Барабан выбрал: ${picked.name}`
              : "Пусть колоду выберет барабан"}
          </span>
        </div>
      </Card>

      <Card title="Условия">
        <Select
          label="Карточек на игрока"
          hint="Все проходят одни и те же карточки в одном порядке — иначе счёт сравнивает везение, а не игроков."
          value={String(cards)}
          onChange={(event) => setCards(Number(event.target.value))}
        >
          {CARD_COUNTS.map((count) => (
            <option key={count} value={count}>
              {count}
            </option>
          ))}
        </Select>

        <Select
          label="Время на карточку"
          value={String(seconds)}
          onChange={(event) => setSeconds(Number(event.target.value))}
        >
          {SECONDS.map((value) => (
            <option key={value} value={value}>
              {value} с
            </option>
          ))}
        </Select>

        {deck && !enough && (
          <p className="text-13 text-danger-text">
            В колоде {deck.card_count} карточек — на дуэль из {cards} не хватит.
          </p>
        )}
      </Card>

      {error && (
        <p className="text-13 text-danger-text" role="alert">
          {error}
        </p>
      )}

      <div className="flex flex-wrap gap-2.5">
        <Button
          disabled={busy || !enough}
          onClick={() =>
            onStart({
              deckId: chosen,
              players: names,
              cards,
              secondsPerCard: seconds,
            })
          }
        >
          Начать дуэль
        </Button>
        <Button variant="ghost" onClick={onLeave}>
          Выйти
        </Button>
      </div>
    </div>
  );
}
