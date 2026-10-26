"""Contrast requirements and platform documentation checks."""

import re
from dataclasses import dataclass, field

from .contract import ANSI, CORE_TABLE, SURFACES, THEMES
from .code_roles import ROLES


HEX = re.compile(r"#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?")


@dataclass
class Report:
    errors: list = field(default_factory=list)
    tokens: int = 0
    contrast_checks: int = 0
    platform_values: int = 0


def contrast_rules(color_names):
    """Required pairs from the token contract and usage notes; no rounded thresholds."""
    rules = {}

    def add(foreground, backgrounds, minimum, themes=THEMES):
        for background in backgrounds:
            for theme in themes:
                key = (foreground, background, theme)
                rules[key] = max(minimum, rules.get(key, 0))

    for name in ("ink", "ink-muted", "accent", "accent-hover", "frost", "fern"):
        add(name, SURFACES, 4.5)
    add("ink-subtle", ["bg-base", "bg-surface", "bg-raised"], 4.5)
    add("line-strong", ["bg-base", "bg-surface"], 3)
    add("focus-ring", SURFACES, 3)
    for foreground, background in (
        ("ink-inverse", "bg-inverse"), ("on-accent", "accent"),
        ("on-accent", "accent-hover"), ("accent-ink", "accent-soft"),
        ("on-danger", "danger"),
    ):
        add(foreground, [background], 4.5)
    for status in ("info", "success", "warning", "danger"):
        add(status, [status + "-soft"], 4.5)
        add("ink", [status + "-soft"], 4.5)
    add("ink", ["selection", "diff-add-bg", "diff-del-bg", "search-match", "search-match-current"], 4.5)
    for name in sorted(color_names):
        if name.startswith("syn-"):
            add(name, ["bg-base", "bg-sunken", "bg-surface"], 4.5)
    add("syn-comment", ["bg-base"], 5)
    add("syn-comment", ["bg-base"], 7.5, ("night",))
    # ANSI 0 is a background role in dark themes, not normal reading text.
    for name in ANSI[1:]:
        add(name, ["bg-base"], 4.5)
    add("ansi-black", ["bg-base"], 4.5, ("day",))
    for role in ROLES.values():
        add(role.color, role.backgrounds, role.minimum)
    return rules


def check_platforms(markdown, resolved, report):
    tables = []
    current = []
    for number, line in enumerate(markdown.splitlines() + [""], 1):
        if line.lstrip().startswith("|"):
            cells = [c.strip().strip("`").strip() for c in line.strip().strip("|").split("|")]
            current.append((number, cells))
        elif current:
            tables.append(current)
            current = []
    found = set()
    for table in tables:
        start, headers = table[0]
        if headers[0] not in ("Token", "#"):
            continue
        kind = "ansi" if headers[0] == "#" else "core"
        prefix = f"platforms.md:{start} ({kind})"
        if kind in found:
            report.errors.append(f"{prefix}: duplicate colour table; keep one")
        found.add(kind)
        expected_headers = ["Token", "Night", "Day", "Aurora"]
        if kind == "ansi":
            expected_headers.insert(0, "#")
        if sorted(headers) != sorted(expected_headers):
            report.errors.append(f"{prefix}: headers must be {', '.join(expected_headers)}")
            continue
        if len(table) < 2 or len(table[1][1]) != len(headers) or any(
            not re.fullmatch(r":?-{3,}:?", cell) for cell in table[1][1]
        ):
            report.errors.append(f"{prefix}: missing or malformed Markdown table separator")
            continue
        required = set(ANSI if kind == "ansi" else CORE_TABLE)
        seen = set()
        for number, cells in table[2:]:
            prefix = f"platforms.md:{number}"
            if len(cells) != len(headers):
                report.errors.append(f"{prefix}: expected {len(headers)} columns, got {len(cells)}")
                continue
            row = dict(zip(headers, cells))
            name = row["Token"]
            if name in seen:
                report.errors.append(f"{prefix}: duplicate row for {name}; keep one")
            seen.add(name)
            if (name, "night") not in resolved or not HEX.fullmatch(resolved[(name, "night")]):
                report.errors.append(f"{prefix}: unknown colour token {name!r}; correct the token name")
                continue
            if kind == "ansi" and (name not in ANSI or row["#"] != str(ANSI.index(name))):
                report.errors.append(f"{prefix}: incorrect ANSI slot for {name}")
            for theme in THEMES:
                expected = resolved[(name, theme)]
                actual = row[theme.title()]
                report.platform_values += 1
                if not HEX.fullmatch(actual) or actual.lower() != expected:
                    report.errors.append(f"{prefix}: {name}[{theme}] is {actual!r}; use {expected} from the resolved DTCG sources")
        for missing in sorted(required - seen):
            report.errors.append(f"platforms.md ({kind}): missing row for {missing}; restore the row")
    for kind in sorted({"core", "ansi"} - found):
        report.errors.append(f"platforms.md: missing {kind} colour table; restore its header and rows")
