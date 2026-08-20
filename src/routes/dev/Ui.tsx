import { useState } from "react";

import { PingCard } from "@/components/PingCard";
import { TimerIcon } from "@/components/nav/icons";
import {
  Button,
  Card,
  Dialog,
  EmptyState,
  Input,
  Select,
  Switch,
} from "@/components/ui";
import type { ButtonSize, ButtonVariant } from "@/components/ui";

/**
 * Витрина примитивов. Только для разработки: маршрут `/dev/ui` регистрируется
 * под `import.meta.env.DEV`, в релизную сборку модуль не попадает.
 *
 * Нужна, чтобы проверять компоненты на 380px и 1400px, не дожидаясь экранов,
 * которые их используют.
 */

const VARIANTS: ButtonVariant[] = ["primary", "secondary", "ghost", "danger"];
const SIZES: ButtonSize[] = ["sm", "md", "lg"];

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-4.5">
      <h2 className="font-mono text-12 tracking-label-2 text-accent-text-2 uppercase">
        {title}
      </h2>
      {children}
    </section>
  );
}

export function Ui() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [sync, setSync] = useState(true);
  const [notify, setNotify] = useState(false);

  return (
    <main className="mx-auto flex max-w-app flex-col gap-8 px-5 py-6.5 sm:px-14 sm:py-11">
      <header className="flex flex-col gap-2.5">
        <h1 className="text-21 font-semibold tracking-title text-text sm:text-30">
          Примитивы
        </h1>
        <p className="text-14 leading-text text-text-muted sm:text-15">
          Компоненты из <code className="font-mono">src/components/ui/</code>.
          Проверять на 380px и 1400px.
        </p>
      </header>

      <Section title="Кнопки — варианты">
        <div className="flex flex-wrap gap-3.5">
          {VARIANTS.map((variant) => (
            <Button key={variant} variant={variant}>
              {variant}
            </Button>
          ))}
        </div>
        <div className="flex flex-wrap gap-3.5">
          {VARIANTS.map((variant) => (
            <Button key={variant} variant={variant} disabled>
              {variant} disabled
            </Button>
          ))}
        </div>
      </Section>

      <Section title="Кнопки — размеры">
        <div className="flex flex-wrap items-center gap-3.5">
          {SIZES.map((size) => (
            <Button key={size} variant="primary" size={size}>
              {size}
            </Button>
          ))}
        </div>
        <Button variant="primary" block>
          block — так кнопки складываются на мобилке
        </Button>
      </Section>

      <Section title="Карточка">
        <div className="grid gap-4.5 sm:grid-cols-2">
          <Card title="Ближайшие вехи" aside="осталось 18 дней">
            <p className="text-14 leading-text text-text-dim">
              Заголовок и приписка справа — необязательные.
            </p>
          </Card>
          <Card>
            <p className="text-14 leading-text text-text-dim">
              Карточка без заголовка.
            </p>
          </Card>
        </div>
      </Section>

      <Section title="Поля">
        <div className="grid gap-4.5 sm:grid-cols-2">
          <Input label="Почта" placeholder="ты@example.com" type="email" />
          <Input
            label="Название предмета"
            defaultValue="Математический анализ"
          />
          <Input
            label="Почта"
            defaultValue="не-почта"
            invalid
            hint="Похоже на опечатку"
          />
          <Select label="Режим таймера" defaultValue="pomodoro">
            <option value="count-up">Счёт вверх</option>
            <option value="count-down">Обратный отсчёт</option>
            <option value="pomodoro">Pomodoro</option>
          </Select>
        </div>
      </Section>

      <Section title="Тумблеры">
        <Card>
          <Switch label="Синхронизация" checked={sync} onChange={setSync} />
          <span className="h-px bg-hairline" />
          <Switch
            label="Уведомления о смене фазы"
            checked={notify}
            onChange={setNotify}
          />
          <span className="h-px bg-hairline" />
          <Switch
            label="Недоступно без аккаунта"
            checked={false}
            onChange={() => {}}
            disabled
          />
        </Card>
      </Section>

      <Section title="Пустое состояние">
        <EmptyState
          icon={<TimerIcon className="size-8" />}
          title="Предметов пока нет"
          description="Заведи первый предмет — с него начнётся статистика."
          action={<Button variant="primary">Добавить предмет</Button>}
        />
      </Section>

      <Section title="Диалог">
        <div className="flex flex-wrap gap-3.5">
          <Button variant="primary" onClick={() => setDialogOpen(true)}>
            Открыть диалог
          </Button>
        </div>
        <Dialog
          open={dialogOpen}
          onClose={() => setDialogOpen(false)}
          title="Тебя не было 42 минуты"
          description="Засчитать это время в сессию или отбросить?"
          footer={
            <>
              <Button block onClick={() => setDialogOpen(false)}>
                Отбросить
              </Button>
              <Button
                variant="primary"
                block
                onClick={() => setDialogOpen(false)}
              >
                Засчитать
              </Button>
            </>
          }
        />
      </Section>

      <Section title="Мост Rust ↔ TypeScript">
        <PingCard />
      </Section>
    </main>
  );
}
