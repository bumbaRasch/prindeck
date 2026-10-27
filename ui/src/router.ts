import type { Screen } from "./types";

let current: Screen = "scan";
let notify: ((screen: Screen) => void) | null = null;

export function initRouter(onChange: (screen: Screen) => void): void {
  notify = onChange;
}

export function setScreen(screen: Screen): Screen {
  const previous = current;
  current = screen;
  if (previous !== screen && notify !== null) {
    notify(screen);
  }
  return current;
}

export function currentScreen(): Screen {
  return current;
}
