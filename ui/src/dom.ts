export interface OptionSpec {
  value: string;
  label: string;
  disabled?: boolean;
}

export function mustQuery<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (element === null) {
    throw new Error(`element ${selector} not found`);
  }
  return element;
}

export function fillSelect(
  select: HTMLSelectElement,
  options: readonly OptionSpec[],
  selected: string | null,
): void {
  const signature = options
    .map((option) => `${option.value}${option.disabled === true ? "!" : ""}`)
    .join("\u0000");

  if (select.dataset["options"] !== signature) {
    select.replaceChildren(
      ...options.map((option) => {
        const element = document.createElement("option");
        element.value = option.value;
        element.textContent = option.label;
        element.disabled = option.disabled === true;
        return element;
      }),
    );
    select.dataset["options"] = signature;
  }

  if (selected !== null) {
    select.value = selected;
  }
}
