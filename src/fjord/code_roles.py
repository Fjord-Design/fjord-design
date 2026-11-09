"""Portable editor and terminal bindings to Fjord's semantic colours."""

from dataclasses import dataclass
from itertools import combinations
from math import dist

from .contract import ANSI
from .color import srgb_to_oklab as oklab
from .dtcg import TokenError


@dataclass(frozen=True)
class Role:
    color: str
    backgrounds: tuple = ()
    minimum: float = 4.5
    marker: str = ""
    emphasis: str = ""

    @property
    def token(self):
        return f"semantic.color.{self.color}"


ROLES = {
    "editor-background": Role("bg-base"),
    "editor-text": Role("ink", ("bg-base", "bg-surface")),
    "editor-current-line": Role("bg-surface"),
    "editor-line-number": Role("ink-subtle", ("bg-base", "bg-surface")),
    "editor-current-line-number": Role("ink", ("bg-base", "bg-surface")),
    "editor-cursor": Role("accent", ("bg-base", "bg-surface"), 3),
    "editor-cursor-text": Role("on-accent", ("accent",)),
    "editor-selection": Role("selection"),
    "editor-selected-text": Role("ink", ("selection",)),
    "editor-search": Role("search-match"),
    "editor-current-search": Role("search-match-current"),
    "editor-search-text": Role("ink", ("search-match", "search-match-current")),
    "diff-added-background": Role("diff-add-bg"),
    "diff-changed-background": Role("warning-soft"),
    "diff-deleted-background": Role("diff-del-bg"),
    "diff-text": Role("ink", ("diff-add-bg", "warning-soft", "diff-del-bg")),
    "diff-added": Role("git-added", ("bg-base", "diff-add-bg"), marker="+ Added"),
    "diff-changed": Role("git-modified", ("bg-base", "warning-soft"), marker="~ Changed"),
    "diff-deleted": Role("git-removed", ("bg-base", "diff-del-bg"), marker="- Deleted"),
    "diagnostic-error": Role("danger", ("bg-base", "bg-surface"), marker="E Error", emphasis="wavy"),
    "diagnostic-warning": Role("warning", ("bg-base", "bg-surface"), marker="W Warning", emphasis="dashed"),
    "diagnostic-info": Role("info", ("bg-base", "bg-surface"), marker="I Info", emphasis="solid"),
    "diagnostic-hint": Role("ink-muted", ("bg-base", "bg-surface"), marker="H Hint", emphasis="dotted"),
    "git-added": Role("git-added", ("bg-base",), marker="A Added"),
    "git-modified": Role("git-modified", ("bg-base",), marker="M Modified"),
    "git-deleted": Role("git-removed", ("bg-base",), marker="D Deleted"),
    "git-renamed": Role("info", ("bg-base",), marker="R Renamed"),
    "git-untracked": Role("ink-muted", ("bg-base",), marker="? Untracked"),
    "git-ignored": Role("git-ignored", ("bg-base",), marker="! Ignored"),
    "git-conflict": Role("danger", ("bg-base",), marker="U Conflict"),
    "terminal-background": Role("bg-base"),
    "terminal-text": Role("ink", ("bg-base",)),
    "terminal-cursor": Role("accent", ("bg-base",), 3),
    "terminal-cursor-text": Role("on-accent", ("accent",)),
    "terminal-selection": Role("selection"),
    "terminal-selected-text": Role("ink", ("selection",)),
    "terminal-hint": Role("ansi-bright-black", ("bg-base",)),
    "prompt-path": Role("frost", ("bg-base",)),
    "prompt-branch": Role("lilac", ("bg-base",)),
    "prompt-environment": Role("aurora", ("bg-base",)),
    "prompt-duration": Role("ink-subtle", ("bg-base",)),
    "prompt-symbol": Role("accent", ("bg-base",)),
    "prompt-error": Role("danger", ("bg-base",), marker="Exit"),
}

for name in ("comment", "keyword", "string", "number", "type", "function", "property",
             "variable", "operator", "punctuation", "regex"):
    ROLES[f"syntax-{name}"] = Role(f"syn-{name}", ("bg-base", "bg-sunken", "bg-surface"),
                                  emphasis="italic" if name in ("comment", "keyword") else "")
ROLES["syntax-text"] = Role("ink", ("bg-base", "bg-sunken", "bg-surface"))
for name in ANSI:
    # Slot 0 is checked as text in Day by the colour contract.
    ROLES[name] = Role(name, ("bg-base",) if name != "ansi-black" else ())

MAIN_SYNTAX = ("comment", "keyword", "string", "number", "type", "function", "property")
SEPARATED_SYNTAX = MAIN_SYNTAX + ("operator", "regex")


def validate_bindings(composition):
    for name, role in ROLES.items():
        token = composition.tokens.get(role.token)
        if token is None or token.get("$type") != "color":
            raise TokenError(f"{name} [{composition.mode}]: expected colour token {role.token}")


def syntax_distances(composition):
    for first, second in combinations(SEPARATED_SYNTAX, 2):
        colors = [composition.values[f"semantic.color.syn-{name}"] for name in (first, second)]
        minimum = 7 if first in MAIN_SYNTAX and second in MAIN_SYNTAX else 5
        yield first, second, 100 * dist(*map(oklab, colors)), minimum


def check_syntax_distances(composition):
    for first, second, distance, minimum in syntax_distances(composition):
        if distance < minimum:
            raise TokenError(f"syn-{first} / syn-{second} [{composition.mode}]: "
                             f"OKLab distance {distance:.6f} is below {minimum}; separate these syntax colours")
