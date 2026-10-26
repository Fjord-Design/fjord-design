"""Deterministic OKLCH/sRGB conversion and Fjord's chroma-reduction fallback."""

from dataclasses import dataclass
import math
import re

from .dtcg import TokenError, number


SEED = re.compile(r"oklch\(([0-9]+(?:\.[0-9]+)?)%\s+([0-9]+(?:\.[0-9]+)?)\s+(-?[0-9]+(?:\.[0-9]+)?)\)")
GAMUT_TOLERANCE = 1e-7
GAMUT_STEPS = 32


def parse_oklch(text):
    match = SEED.fullmatch(text) if isinstance(text, str) else None
    if match is None:
        raise TokenError(f"invalid Fjord OKLCH seed {text!r}; use oklch(L% C H)")
    result = tuple(map(float, match.groups()))
    value = (result[0] / 100, result[1], result[2])
    validate_oklch(value)
    return value


def design_seed_values(source):
    if (not isinstance(source, dict) or set(source) != {"primary", "accents"}
            or not isinstance(source["primary"], list) or len(source["primary"]) != 3
            or not isinstance(source["accents"], list) or len(source["accents"]) != 4):
        raise TokenError("designSeeds: retain three primary and four accent OKLCH references")
    values = source["primary"] + source["accents"]
    for value in values:
        parse_oklch(value)
    return values


def validate_oklch(value):
    if (not isinstance(value, (tuple, list)) or len(value) != 3
            or not all(number(v) for v in value)
            or not 0 <= value[0] <= 1 or not 0 <= value[1] <= 1):
        raise TokenError("Fjord OKLCH requires finite L in [0,1], C in [0,1] and hue in degrees")


def srgb_to_oklab(color):
    """Use the published linear-sRGB matrices from https://bottosson.github.io/posts/oklab/."""
    r, g, b = [v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4
               for v in color["components"]]
    l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b) ** (1 / 3)
    m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b) ** (1 / 3)
    s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b) ** (1 / 3)
    return (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s)


def srgb_to_oklch(color):
    lightness, a, b = srgb_to_oklab(color)
    chroma = math.hypot(a, b)
    return (lightness, chroma, math.degrees(math.atan2(b, a)) % 360 if chroma > 1e-7 else 0)


def oklch_to_srgb(value):
    """Return encoded sRGB components, including out-of-gamut values."""
    validate_oklch(value)
    lightness, chroma, hue = value
    a, b = chroma * math.cos(math.radians(hue % 360)), chroma * math.sin(math.radians(hue % 360))
    l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (lightness - 0.0894841775 * a - 1.2914855480 * b) ** 3
    linear = (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
              -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
              -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)
    return tuple(12.92 * v if abs(v) <= 0.0031308 else
                 math.copysign(1.055 * abs(v) ** (1 / 2.4) - 0.055, v) for v in linear)


def in_srgb_gamut(components):
    return all(-GAMUT_TOLERANCE <= v <= 1 + GAMUT_TOLERANCE for v in components)


@dataclass(frozen=True)
class MappedColor:
    color: dict
    chroma: float
    mapped: bool


def map_oklch_to_srgb(value):
    """Hold L and hue, reducing C by a fixed 32-step binary search when necessary."""
    raw = oklch_to_srgb(value)
    lightness, chroma, hue = value
    mapped = not in_srgb_gamut(raw)
    if lightness in (0, 1):
        raw, chroma, mapped = (lightness,) * 3, 0, chroma != 0
    elif mapped:
        lower, upper = 0, chroma
        for _ in range(GAMUT_STEPS):
            middle = (lower + upper) / 2
            if in_srgb_gamut(oklch_to_srgb((lightness, middle, hue))):
                lower = middle
            else:
                upper = middle
        chroma = lower
        raw = oklch_to_srgb((lightness, chroma, hue))
    return MappedColor({"colorSpace": "srgb", "components": [min(1, max(0, v)) for v in raw], "alpha": 1},
                       chroma, mapped)
