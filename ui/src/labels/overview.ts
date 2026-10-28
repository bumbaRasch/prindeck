export function jamCountText(count: number): string {
  return count === 0
    ? "замятия: нет"
    : `замятия: ${count} — проверьте тракт бумаги`;
}

export function drumPagesLeftText(pages: number): string {
  return `барабан: осталось ≈${pages} стр.`;
}

export function tonerPagesLeftText(pages: number): string {
  return `≈ ${pages} стр.`;
}
