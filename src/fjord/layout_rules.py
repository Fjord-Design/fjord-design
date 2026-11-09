"""Cross-token safety constraints for layout, interaction and motion."""

from .dtcg import TokenError


def dimension_in_px(value):
    return value["value"] * (16 if value["unit"] == "rem" else 1)


def check_layout(values):
    def value(group, name):
        return dimension_in_px(values[f"semantic.{group}.{name}"])

    for path, item in values.items():
        if path.startswith(("semantic.size.", "semantic.breakpoint.", "semantic.layout.", "semantic.density.")):
            if dimension_in_px(item) < 0:
                raise TokenError(f"{path}: layout dimensions must not be negative")
            if path.startswith(("semantic.size.", "semantic.breakpoint.")) and dimension_in_px(item) == 0:
                raise TokenError(f"{path}: sizes and breakpoints must be positive")
    for name in ("target-minimum", "control-compact", "control-comfortable"):
        if value("size", name) < 24:
            raise TokenError(f"semantic.size.{name}: interaction targets must be at least 24 CSS pixels")
    minimum = value("size", "target-minimum")
    for name in ("control-compact", "control-comfortable", "target-touch"):
        if value("size", name) < minimum:
            raise TokenError(f"semantic.size.{name}: must meet the declared target-minimum")
    if value("size", "target-touch") < 44:
        raise TokenError("semantic.size.target-touch: coarse-pointer targets must be at least 44 CSS pixels")
    if value("size", "control-comfortable") < value("size", "control-compact"):
        raise TokenError("semantic.size: comfortable controls must not be smaller than compact controls")
    for name in ("gap", "padding"):
        if value("density", name + "-compact") > value("density", name + "-comfortable"):
            raise TokenError(f"semantic.density.{name}: compact must not exceed comfortable")
    widths = [value("breakpoint", "breakpoint-" + name) for name in ("sm", "md", "lg")]
    if not 0 < widths[0] < widths[1] < widths[2]:
        raise TokenError("semantic.breakpoint: require positive, increasing sm, md and lg widths")
    if value("border-width", "border-focus") < 2:
        raise TokenError("semantic.border-width.border-focus: focus indicators must be at least 2 CSS pixels")
    if dimension_in_px(values["component.button.primary.focus-width"]) < 2:
        raise TokenError("component.button.primary.focus-width: focus indicators must be at least 2 CSS pixels")
    levels = [values["semantic.z-index.z-" + name] for name in ("base", "sticky", "popover", "modal", "toast", "tooltip")]
    if any(first >= second for first, second in zip(levels, levels[1:])):
        raise TokenError("semantic.z-index: require base < sticky < popover < modal < toast < tooltip")
    durations = [values["semantic.duration.duration-" + name] for name in ("none", "fast", "base", "slow")]
    milliseconds = [item["value"] * (1000 if item["unit"] == "s" else 1) for item in durations]
    if not milliseconds[0] == 0 < milliseconds[1] <= milliseconds[2] <= milliseconds[3] <= 300:
        raise TokenError("semantic.duration: require none = 0 < fast <= base <= slow <= 300ms")
