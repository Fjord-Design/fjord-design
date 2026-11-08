#!/bin/sh
printf '\033]0;Fjord M4 isolated native check\007'
printf 'FJORD NATIVE TERMINAL — explicit ANSI, truecolour, links and status\n\n'
for code in 30 31 32 33 34 35 36 37 90 91 92 93 94 95 96 97; do
 printf '\033[%smSGR %s \033[0m' "$code" "$code"
done
printf '\n'
for code in 40 41 42 43 44 45 46 47 100 101 102 103 104 105 106 107; do
 printf '\033[%sm    \033[0m' "$code"
done
printf '\n\n\033[38;2;140;180;200m24-bit truecolour example\033[0m\n'
printf '\033]8;;https://example.invalid/fjord\033\\Named OSC 8 hyperlink (not opened)\033]8;;\033\\\n'
printf '\033[32m+ Added\033[0m | \033[33m~ Modified\033[0m | \033[31m- Deleted\033[0m\n'
printf '\033[31mE Error: destination unavailable\033[0m\n'
printf '\033[90mFull-opacity bright-black hint\033[0m\n'
printf '\033[1;31mBold normal red\033[0m | \033[91mExplicit bright red\033[0m\n'
printf 'Cursor and selection: select this sentence with the native terminal.\n'
printf 'Prompt context: path ./fjord | branch main | env local | 12ms >\n'
printf '\nThe test process stays open until its owner closes it.\n'
read -r fjord_finish
