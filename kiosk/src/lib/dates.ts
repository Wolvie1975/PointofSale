// Dates travel to and from Rust as local 'YYYY-MM-DD' strings (chrono NaiveDate).

/** Local date → 'YYYY-MM-DD' (not toISOString, which converts to UTC). */
export function toIso(d: Date): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** 'YYYY-MM-DD' → local Date at midnight. */
export function fromIso(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number);
  return new Date(y, m - 1, d);
}

/** Today plus `n` days, as 'YYYY-MM-DD'. */
export function daysFromToday(n: number): string {
  const d = new Date();
  d.setDate(d.getDate() + n);
  return toIso(d);
}

/** 'Saturday, November 21' */
export const longDate = (iso: string) =>
  fromIso(iso).toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });

/** 'Sat' */
export const shortWeekday = (iso: string) => fromIso(iso).toLocaleDateString('en-US', { weekday: 'short' });

/** 'November 2026' */
export const monthTitle = (year: number, month: number) =>
  new Date(year, month - 1, 1).toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
