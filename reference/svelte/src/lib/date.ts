export function parseDate(value: string): Date {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value))
    throw new Error("Use a valid YYYY-MM-DD date.");
  const [year, month, day] = value.split("-").map(Number);
  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  date.setUTCHours(0, 0, 0, 0);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  )
    throw new Error("Use a real calendar date.");
  return date;
}
export function isoDate(value: Date): string {
  return `${String(value.getUTCFullYear()).padStart(4, "0")}-${String(value.getUTCMonth() + 1).padStart(2, "0")}-${String(value.getUTCDate()).padStart(2, "0")}`;
}
export function addDays(value: string, count: number): string {
  const date = parseDate(value);
  date.setUTCDate(date.getUTCDate() + count);
  return isoDate(date);
}
export function addMonths(value: string, count: number): string {
  const date = parseDate(value),
    day = date.getUTCDate();
  date.setUTCDate(1);
  date.setUTCMonth(date.getUTCMonth() + count);
  const end = new Date(date);
  end.setUTCMonth(end.getUTCMonth() + 1);
  end.setUTCDate(0);
  date.setUTCDate(Math.min(day, end.getUTCDate()));
  return isoDate(date);
}
export function todayDate(): string {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}
