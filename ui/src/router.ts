import { mustQuery } from "./dom";
import { SCREEN_TITLES } from "./labels";
import type { Screen } from "./types";

const SCREENS: readonly Screen[] = ["scan", "overview"];

function isScreen(value: string | undefined): value is Screen {
  return value === "scan" || value === "overview";
}

let current: Screen = "scan";
let notify: ((screen: Screen) => void) | null = null;
let screenTitle: HTMLHeadingElement;
const sections = new Map<Screen, HTMLElement>();
const navItems = new Map<Screen, HTMLButtonElement>();

function apply(screen: Screen): void {
  for (const [name, section] of sections) {
    section.hidden = name !== screen;
  }
  for (const [name, item] of navItems) {
    const active = name === screen;
    item.tabIndex = active ? 0 : -1;
    if (active) {
      item.setAttribute("aria-current", "page");
    } else {
      item.removeAttribute("aria-current");
    }
  }
  screenTitle.textContent = SCREEN_TITLES[screen];
}

function moveFocusedNav(from: Screen, offset: number): void {
  const index = SCREENS.indexOf(from);
  const next = SCREENS[(index + offset + SCREENS.length) % SCREENS.length];
  if (next === undefined) {
    return;
  }
  const target = navItems.get(next);
  if (target === undefined) {
    return;
  }
  target.tabIndex = 0;
  const previous = navItems.get(from);
  if (previous !== undefined) {
    previous.tabIndex = -1;
  }
  target.focus();
}

export function initRouter(onChange: (screen: Screen) => void): void {
  notify = onChange;
  screenTitle = mustQuery<HTMLHeadingElement>("#screen-title");

  for (const node of document.querySelectorAll<HTMLElement>("[data-screen]")) {
    const name = node.dataset["screen"];
    if (isScreen(name)) {
      sections.set(name, node);
    }
  }

  const navList = mustQuery<HTMLElement>(".nav-list");
  for (const node of document.querySelectorAll<HTMLButtonElement>("[data-nav]")) {
    const name = node.dataset["nav"];
    if (!isScreen(name)) {
      continue;
    }
    navItems.set(name, node);
    node.addEventListener("click", () => {
      setScreen(name);
    });
  }

  navList.addEventListener("keydown", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLButtonElement)) {
      return;
    }
    const name = target.dataset["nav"];
    if (!isScreen(name)) {
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveFocusedNav(name, 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      moveFocusedNav(name, -1);
    }
  });

  apply(currentScreen());
}

export function setScreen(screen: Screen): Screen {
  const previous = current;
  current = screen;
  apply(screen);
  screenTitle.focus();
  if (previous !== screen && notify !== null) {
    notify(screen);
  }
  return current;
}

export function currentScreen(): Screen {
  return current;
}
