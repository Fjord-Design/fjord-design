#!/bin/sh
# Run with: sh adapters/vscode/fixtures/ansi.sh
# Labels preserve the meaning when colours cannot be distinguished.
printf 'Fjord terminal: cursor, selection and all sixteen ANSI slots\n'
slot=0
for name in black red green yellow blue magenta cyan white; do
  printf '\033[%sm%-2s %-10s readable context\033[0m\n' "$((30 + slot))" "$slot" "$name"
  slot=$((slot + 1))
done
slot=8
for name in black red green yellow blue magenta cyan white; do
  printf '\033[%sm%-2s bright %-10s readable context\033[0m\n' "$((90 + slot - 8))" "$slot" "$name"
  slot=$((slot + 1))
done
printf '\033[36m~/fjord\033[0m \033[35mmain\033[0m \033[32mA Added\033[0m \033[33mM Modified\033[0m \033[31mD Deleted\033[0m\n'
printf 'Search for Fjord, select these lines, then compare the cursor and text.\n'
