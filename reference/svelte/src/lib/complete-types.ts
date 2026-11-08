export type DisclosureItem = {
  value: string;
  label: string;
  disabled?: boolean;
};
export type NavigationItem = {
  label: string;
  href: string;
  current?: boolean;
  disabled?: boolean;
};
export type SelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
  group?: string;
};
export type MenuItem = {
  id: string;
  label: string;
  disabled?: boolean;
  shortcut?: string;
  kind?: "action" | "checkbox" | "radio";
  checked?: boolean;
  group?: string;
  children?: MenuItem[];
};
export type DataColumn = { key: string; label: string; sortable?: boolean };
export type DataRow = {
  id: string;
  [key: string]: string | number | boolean | null;
};
export type ChartDatum = { label: string; value: number };
export type TraceStep = {
  id: string;
  label: string;
  status: "pending" | "running" | "complete" | "error";
  startedAt?: string;
  durationMs?: number;
  error?: string;
  detail?: string;
};
export type Question = {
  id: string;
  title: string;
  description?: string;
  validate?: (answer: string) => string | undefined;
};
export type CarouselSlide = { id: string; label: string };
