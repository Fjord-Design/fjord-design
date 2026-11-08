// Fjord: comments remain readable during selection and search.
type ThemeName = "Night" | "Day" | "Aurora";

export function greet(name: string, theme: ThemeName = "Night"): string {
  const count = 42;
  const valid = /^[A-Za-z]+$/.test(name);
  return valid ? `${theme}: hello ${name} (${count})` : "Choose a name";
}

// Deliberate type error: inspect Zed's diagnostic message and underline.
const invalid: number = "Fjord";
console.log(greet("Nordlys"), invalid);
