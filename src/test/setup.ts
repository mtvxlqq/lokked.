import "@testing-library/jest-dom/vitest";

import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

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
