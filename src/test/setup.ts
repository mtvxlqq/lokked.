import "@testing-library/jest-dom/vitest";

import { cleanup, configure } from "@testing-library/react";
import { afterEach } from "vitest";

// Ожиданию `findBy*` по умолчанию отводится секунда. Тестам с поддельными
// таймерами (`shouldAdvanceTime`) её не хватает, когда файлы идут
// параллельно и каждому воркеру достаётся доля ядра: секунда истекает
// раньше, чем React успевает перерисоваться. Пять секунд ничего не
// замедляют — ожидание всё равно заканчивается на первом совпадении.
configure({ asyncUtilTimeout: 5000 });

// React Testing Library does not auto-clean when `globals` is on in some
// configurations; doing it explicitly keeps tests independent.
afterEach(() => {
  cleanup();
});

// jsdom parses `<dialog>` but implements neither `showModal` nor `close`, so
// any component built on the native dialog throws on mount. These stand-ins
// only maintain the `open` attribute — the parts that matter to a test are
// what is rendered and whether `close` fires the `close` event; the modal
// behaviour itself (focus trap, backdrop, Esc) is the browser's and is not
// what these tests are checking.
if (!HTMLDialogElement.prototype.showModal) {
  HTMLDialogElement.prototype.showModal = function showModal(
    this: HTMLDialogElement,
  ) {
    this.open = true;
  };
  HTMLDialogElement.prototype.show = function show(this: HTMLDialogElement) {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function close(this: HTMLDialogElement) {
    this.open = false;
    this.dispatchEvent(new Event("close"));
  };
}
