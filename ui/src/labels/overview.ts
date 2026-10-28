export function jamCountText(count: number): string {
  return count === 0 ? "замятий не было" : `всего замятий: ${count}`;
}

export function drumPagesLeftText(pages: number): string {
  return `барабан: осталось ≈${pages} стр.`;
}

export function tonerPagesLeftText(pages: number): string {
  return `≈ ${pages} стр.`;
}
