export type Mode = "night" | "day" | "aurora";
export type ModePreference = Mode | "system";
export type RadioOption = { value: string; label: string; disabled?: boolean };
export type TabItem = { value: string; label: string; disabled?: boolean };
export type FieldControl = {
  id: string;
  required: boolean;
  disabled: boolean;
  "aria-invalid": "true" | undefined;
  "aria-describedby": string | undefined;
};
