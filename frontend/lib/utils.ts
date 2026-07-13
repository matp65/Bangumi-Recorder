import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(value: string | null | undefined, withTime = false) {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T"));
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: withTime ? "2-digit" : undefined,
    minute: withTime ? "2-digit" : undefined,
    hour12: false,
  }).format(date);
}

export function formatUnknown(value: unknown, pretty = false): string {
  if (value === null || value === undefined) return "—";
  if (["string", "number", "boolean"].includes(typeof value)) return String(value);
  return JSON.stringify(value, null, pretty ? 2 : undefined);
}
