/**
 * Observe a mode preference without changing document globals or storing a choice.
 * @param {import('./types.js').ModePreference} preference
 * @param {(mode: import('./types.js').Mode) => void} onChange
 * @returns {() => void} Remove the system listener when the preference changes.
 */
export function watchMode(preference, onChange) {
  if (!["night", "day", "aurora", "system"].includes(preference)) {
    throw new Error("Choose night, day, aurora or system.");
  }
  if (preference !== "system") {
    onChange(preference);
    return () => {};
  }
  if (typeof window === "undefined") {
    onChange("night");
    return () => {};
  }
  const query = window.matchMedia("(prefers-color-scheme: light)");
  const update = () => onChange(query.matches ? "day" : "night");
  update();
  query.addEventListener("change", update);
  return () => query.removeEventListener("change", update);
}
