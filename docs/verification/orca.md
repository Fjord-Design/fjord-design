# Orca and Firefox verification

Recorded 8 November 2026 for the complete-library candidate. This record supplements [M3/M4 verification](m3-m4.md).

## Target and isolation

The owner authorized an isolated Orca installation and native command checks. Twenty missing packages were downloaded from the configured repositories and extracted under `dist/tools/orca-51.1/prefix`. Archive SHA-256 values matched the cached package database. No system package installation or vendor code patch was used. The local `packages.json` records the package versions, URLs and checksums.

| Tool | Observed version |
| --- | --- |
| Orca | 51.1 |
| Firefox | 159.0, build 20261027034604 |
| Speech Dispatcher / eSpeak NG | 0.12.1 / 1.52.0 |
| AT-SPI / system Python | 2.62.1 / 3.14.8 |
| PipeWire | 1.6.10 |
| Desktop / kernel | KDE Plasma 6.8.2 Wayland, isolated XWayland Firefox / 7.3.1-1-cachyos |

The test uses private session and accessibility buses, temporary Firefox and Orca settings, and an owned audio sink. Input targets the owned Firefox window. The recorder captures only that sink's monitor. The default sink and system package list remain unchanged. The project still uses its pinned Python 3.15.0 for project checks.

Speech uses PipeWire with `PIPEWIRE_NODE` set to the owned sink, `PIPEWIRE_RUNTIME_DIR` set to the host runtime directory, and default-sink fallback disabled. The private `XDG_RUNTIME_DIR` remains separate. These are supported [PipeWire process settings](https://docs.pipewire.org/page_man_pipewire_1.html).

## Scope and findings

All 59 distinct fixture cases passed after focused reruns: 39 keyboard and announcement cases, 18 page-reading cases, and two Firefox touch-event cases. The final `summary.json` maps each case to its successful evidence directory. Earlier failures remain recorded and are not counted as passes.

The final checks also require completed synthesis of the date label, table sort direction, questionnaire validation, composed required-selection error and attachment retry description. A separate screenshot check confirmed visible questionnaire errors without horizontal overflow. Normal exit, SIGINT and SIGTERM released the owned native resources and preserved the default sink. The native SIGTERM probe exited with the expected status 143 and a successful cleanup record.

Native checks led to these component changes:

- DatePicker's anonymous native date segments lacked the field name in Orca. A named group now supplies that context while retaining the native input and label.
- DataTable announced the sort arrow without its direction. Its existing status region now names the sorted column and direction.
- Questionnaire exposed an invalid field without announcing the validation message. Its persistent alert now declares assertive, atomic updates explicitly.
- Orca skipped an attachment error during continuous reading because its alert text produced an elements/voices mismatch. Alert containers now retain paragraph children in Attachment, DataTable and Combobox. Questionnaire retains direct text in its persistent alert because the nested variant suppressed its live announcement on this target. Attachment Retry also references the visible error as its description.

The keyboard cases cover all eleven new component families, all seven MVP families and the three composed workspace tabs. They check names, values, expanded/checked states, error messages, live status, keyboard changes and focus return. Family checks use Night. Composed workspace checks use Day. Fixtures expect English announcements. Programmatic focus selects each starting control. Subsequent keyboard input and state changes occur in native Firefox.

The page-reading cases check selected static labels in each family. They require completed Speech Dispatcher messages, including the expected text. They do not establish exhaustive reading order for every state.

The touch cases check HoverCard tap toggling and Drawer drag dismissal. Firefox receives WebDriver touch actions. The page must report trusted events with `pointerType="touch"` and the expected component state. This is browser touch emulation, not a physical touchscreen test.

Orca's [debug output](https://orca.gnome.org/debugging) records accessibility events and generated announcements. Speech Dispatcher protocol logs separately establish completed synthesis. Non-silent audio confirms output at the isolated sink. These checks do not establish human listening comprehension, Braille operation, another screen reader, or a full browser/version matrix.

## Test setup findings

Earlier failed probes remain in the ignored evidence directory. Structural-navigation key injection was unreliable in this XWayland setup. Automatic reading also interfered with focus tests until the harness stopped it before each interaction. Some page-reading expectations incorrectly required group names that Orca omits during continuous reading. Those cases now check visible content, while focus cases check the group context. One run overlapped a development-server rebuild and reloaded during interactions. A later case stopped when the owned Firefox window lost focus, then passed on rerun. Subsequent batches used a static catalogue, including an immutable copy during concurrent build checks.

The Pulse compatibility backend synthesized initial audio but did not send completion events. Later phrases remained queued and continuous reading stalled. The already installed native PipeWire backend delivered completion events and continued reading. No dependency change was needed.

The initial transcript matcher also omitted continuous reading, which uses a different Orca output path. The final page-reading checks match completed Speech Dispatcher messages, rather than speech-generation plans or queued text.

## Repeat the checks

Use existing permitted native tools and a prepared Orca 51.1 prefix. The helper installs nothing. This prefix uses the host Python 3.14 bindings. The required native tools include Firefox, XWayland, AT-SPI, PipeWire, `pactl`, `parec`, `xdotool`, DBus, GSettings and ImageMagick.

Build Storybook from the repository root. Serve a stable copy while native checks run:

```sh
mise run storybook:build
/usr/bin/python3 -m http.server 6008 --bind 127.0.0.1 --directory dist/storybook
```

In another terminal, run:

```sh
/usr/bin/python3 scripts/check_orca.py \
  --prefix dist/tools/orca-51.1/prefix \
  --base-url http://127.0.0.1:6008 \
  --output-dir dist/verification/orca-new-run
```

Use a new empty output directory to preserve earlier evidence. Do not rebuild the served directory during a run. The helper records case results, runtime metadata, audio, protocol logs and cleanup results. Review failures before accepting the result. Missing native tools are not passes. These display-dependent checks remain separate from headless CI.

Local evidence for this session is under `dist/verification/orca-2026-11-08/`. Generated recordings, installed tools, profiles and browser artifacts are not committed.
