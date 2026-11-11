"""Required token roles, mode names and component bindings."""

MODES = ("night", "day", "aurora")
SURFACES = (
    ["bg-sunken", "bg-base", "bg-surface", "bg-raised", "bg-hover", "bg-pressed", "bg-selected"]
)
ANSI = ["ansi-black", "ansi-red", "ansi-green", "ansi-yellow", "ansi-blue", "ansi-magenta", "ansi-cyan", "ansi-white", "ansi-bright-black", "ansi-bright-red", "ansi-bright-green", "ansi-bright-yellow", "ansi-bright-blue", "ansi-bright-magenta", "ansi-bright-cyan", "ansi-bright-white"]
REQUIRED = {
    "color": set(
        ["bg-sunken", "bg-base", "bg-surface", "bg-raised", "bg-hover", "bg-pressed", "bg-selected", "bg-inverse", "line", "line-strong", "ink", "ink-muted", "ink-subtle", "ink-disabled", "ink-inverse", "accent", "accent-hover", "accent-pressed", "accent-soft", "accent-ink", "on-accent", "focus-ring", "selection", "frost", "aurora", "lilac", "sage", "fern", "latte", "foam", "rose", "info", "success", "warning", "danger", "hearth", "hearth-hover", "hearth-soft", "lilac-soft", "info-soft", "success-soft", "warning-soft", "danger-soft", "on-danger", "syn-comment", "syn-keyword", "syn-string", "syn-function", "syn-type", "syn-variable", "syn-property", "syn-number", "syn-operator", "syn-punctuation", "syn-regex", "git-added", "git-modified", "git-removed", "git-ignored", "diff-add-bg", "diff-del-bg", "search-match", "search-match-current", "chart-1", "chart-2", "chart-3", "chart-4", "chart-5", "chart-6", "chart-grid", "scrim"]
        + ANSI
    ),
    "font-size": {"text-2xs", "text-xs", "text-sm", "text-md", "text-base", "text-lg", "text-xl", "text-2xl", "text-3xl"},
    "spacing": {"step-1", "step-2", "step-3", "step-4", "step-5", "step-6", "step-8", "step-10", "step-12", "step-16"},
    "radius": {"none", "window", "xs", "panel", "control", "full"},
    "shadow": {"shadow-sm", "shadow-raised", "shadow-overlay", "shadow-pressed"},
    "border-width": {"border-hairline", "border-focus", "border-active"},
    "duration": {"duration-none", "duration-fast", "duration-base", "duration-slow"},
    "easing": {"easing-standard", "easing-exit"},
    "z-index": {"z-base", "z-sticky", "z-popover", "z-modal", "z-toast", "z-tooltip"},
    "size": {"control-compact", "control-comfortable", "target-minimum", "target-touch", "icon-control", "icon-status", "icon-medium", "icon-default", "icon-empty"},
    "breakpoint": {"breakpoint-sm", "breakpoint-md", "breakpoint-lg"},
    "layout": {"content-max", "focus-offset", "page-gutter", "page-gutter-wide"},
    "density": {"gap-comfortable", "padding-comfortable", "gap-compact", "padding-compact"},
}


GROUP_TYPES = {
    "color": "color",
    "font-size": "dimension",
    "spacing": "dimension",
    "radius": "dimension",
    "shadow": "shadow",
    "border-width": "dimension",
    "duration": "duration",
    "easing": "cubicBezier",
    "z-index": "number",
    "size": "dimension",
    "breakpoint": "dimension",
    "layout": "dimension",
    "density": "dimension",
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
    roles = {
        f"semantic.{group}.{name}": GROUP_TYPES[group]
        for group, names in REQUIRED.items()
        for name in names
    }
    roles.update(
        {
            f"component.button.primary.{name}": kind
            for name, kind in COMPONENT_TYPES.items()
        }
    )
    roles.update(
        {
            f"semantic.typography.{name}": "typography"
            for name in ["display", "title-1", "title-2", "title-3", "title-4", "lead", "body", "body-strong", "label", "small", "caption", "overline", "code", "code-small", "code-italic"]
        }
    )
    roles.update(
        {f"semantic.font.{name}": "fontFamily" for name in ("sans", "display", "mono")}
    )
    return roles


def mode_roles():
    return {
        f"semantic.{group}.{name}"
        for group in ("color", "shadow")
        for name in REQUIRED[group]
    }
