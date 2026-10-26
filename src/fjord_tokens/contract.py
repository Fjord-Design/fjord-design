"""Required token roles, mode names and component bindings."""

THEMES = ("night", "day", "aurora")
SURFACES = "bg-sunken bg-base bg-surface bg-raised bg-hover bg-active".split()
ANSI = ("ansi-black ansi-red ansi-green ansi-yellow ansi-blue ansi-magenta "
        "ansi-cyan ansi-white ansi-bright-black ansi-bright-red ansi-bright-green "
        "ansi-bright-yellow ansi-bright-blue ansi-bright-magenta ansi-bright-cyan "
        "ansi-bright-white").split()
CORE_TABLE = ("bg-base bg-surface bg-raised bg-sunken line ink ink-muted ink-subtle "
              "accent on-accent selection syn-comment syn-keyword syn-string "
              "syn-function syn-type syn-property syn-number").split()
REQUIRED = {
    "color": set((
        "bg-sunken bg-base bg-surface bg-raised bg-hover bg-active bg-inverse "
        "line line-strong ink ink-muted ink-subtle ink-disabled ink-inverse "
        "accent accent-hover accent-soft accent-ink on-accent focus-ring selection "
        "frost aurora lilac sage fern latte foam rose info success warning danger "
        "info-soft success-soft warning-soft danger-soft on-danger "
        "syn-comment syn-keyword syn-string syn-function syn-type syn-variable "
        "syn-property syn-number syn-operator syn-punctuation syn-regex "
        "git-added git-modified git-removed git-ignored diff-add-bg diff-del-bg "
        "search-match search-match-current chart-1 chart-2 chart-3 chart-4 "
        "chart-5 chart-6 chart-grid scrim"
    ).split() + ANSI),
    "font-size": set("text-2xs text-xs text-sm text-md text-base text-lg text-xl text-2xl text-3xl".split()),
    "spacing": set("space-1 space-2 space-3 space-4 space-5 space-6 space-8 space-10 space-12 space-16".split()),
    "radius": set("radius-none radius-window radius-xs radius-panel radius-control radius-full".split()),
    "shadow": set("shadow-sm shadow-raised shadow-overlay shadow-pressed".split()),
    "border-width": set("border-hairline border-focus border-active".split()),
    "duration": set("duration-none duration-fast duration-base duration-slow".split()),
    "easing": set("easing-standard easing-exit".split()),
    "z-index": set("z-base z-sticky z-popover z-modal z-toast z-tooltip".split()),
    "size": set("control-compact control-comfortable target-minimum target-touch icon-control icon-status icon-medium icon-default icon-empty".split()),
    "breakpoint": set("breakpoint-sm breakpoint-md breakpoint-lg".split()),
    "layout": set("content-max focus-offset page-gutter page-gutter-wide".split()),
    "density": set("gap-comfortable padding-comfortable gap-compact padding-compact".split()),
}


GROUP_TYPES = {
    "color": "color", "font-size": "dimension", "spacing": "dimension",
    "radius": "dimension", "shadow": "shadow", "border-width": "dimension",
    "duration": "duration", "easing": "cubicBezier", "z-index": "number",
    "size": "dimension", "breakpoint": "dimension", "layout": "dimension", "density": "dimension",
}
COMPONENT_TYPES = {
    "background": "color",
    "foreground": "color",
    "hover-background": "color",
    "pressed-background": "color",
    "disabled-foreground": "color",
    "focus-color": "color",
    "focus-width": "dimension",
    "radius": "dimension",
    "padding-inline": "dimension",
    "padding-block": "dimension",
    "font-size": "dimension",
    "font-family": "fontFamily",
}


def required_roles():
    roles = {f"semantic.{group}.{name}": GROUP_TYPES[group]
             for group, names in REQUIRED.items() for name in names}
    roles.update({f"component.button.primary.{name}": kind for name, kind in COMPONENT_TYPES.items()})
    roles.update({f"semantic.font.{name}": "fontFamily" for name in ("sans", "display", "mono")})
    return roles


def mode_roles():
    return {f"semantic.{group}.{name}" for group in ("color", "shadow") for name in REQUIRED[group]}
