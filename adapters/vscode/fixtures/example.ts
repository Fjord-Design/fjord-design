// Fjord: comments, properties and selected text stay readable in each mode.
interface Theme {
  name: string;
  enabled: boolean;
}

const themes: ReadonlyArray<Theme> = [
  { name: "Night", enabled: true },
  { name: "Day", enabled: true },
  { name: "Aurora", enabled: true },
];

export function greet(theme: Theme, count = 42): string {
  const valid = /^[A-Za-z]+$/.test(theme.name);
  return valid ? `${theme.name}: Fjord (${count})` : "Choose a theme";
}

// Deliberate error for the built-in TypeScript service and Problems panel.
const invalid: number = "Fjord";
console.log(themes.map(theme => greet(theme)), invalid);
