# Дизайн-материалы

Макеты сделаны в Claude Design, проект «Zen mode design prototype»
(`claude.ai/design/p/735a59bb-e0ce-416d-b983-83a3de2ec07b`).

| Файл | Что это |
|---|---|
| `tokens.md` | Дизайн-токены: цвета, типографика, радиусы, отступы, свечения, анимация. Единственный источник правды для `src/styles/tokens.css`. |
| `mockups/lokked-zen.html` | Исходник макета. Открывается в браузере как есть — рядом лежащий `support.js` подтягивает React с CDN и рендерит разметку, так что для просмотра нужен интернет. |
| `mockups/support.js` | Рантайм Claude Design. Сгенерирован, не редактировать. |
| `screens/*.png` | Скриншоты макета по разделам, сняты с `mockups/lokked-zen.html`. |

## Разделы макета

| Скриншот | Что внутри |
|---|---|
| `screens/splash.png` | Экран запуска, desktop и mobile |
| `screens/screens-2-6.png` | Главный экран, активный Pomodoro-таймер, карточка «Классика», карточка «Блиц», статистика — каждый в desktop и mobile |
| `screens/zen-mode.png` | Zen-режим: активное и приглушённое состояние, desktop и mobile |
| `screens/m18-m22.png` | Стрик, локальная блиц-дуэль, аккаунт, группы, онлайн-блиц (этапы M18—M22) |
| `screens/logo-directions.png` | Шесть направлений логотипа |

**Код макетов — референс, а не исходник.** В нём захардкоженные цвета, выдуманные
данные и произвольные значения вместо токенов. Экраны реализуются заново на токенах
из `src/styles/tokens.css` и примитивах из `src/components/ui/`; данные берутся только
из tauri-команд. Копировать разметку макета в `src/` нельзя.

Каталог не попадает в сборку: Vite собирает только то, на что есть импорт из `src/`.

## Пересъёмка скриншотов

```sh
google-chrome --headless --disable-gpu --hide-scrollbars \
  --virtual-time-budget=25000 --window-size=3300,12000 \
  --screenshot=raw.png docs/designs/mockups/lokked-zen.html
magick raw.png -bordercolor '#0A0A0B' -border 1 -fuzz 1% -trim +repage out.png
```

Чтобы снять один раздел, добавь в `<head>` копии макета правило
`body section:nth-of-type(N) { display:none }` для всех остальных N.
