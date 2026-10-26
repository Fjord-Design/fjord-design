"""Validate semantic usage, state bindings and their required contrast pairs."""

import re

from .dtcg import TokenError, number, read_json


ROLE_NAMES = set('canvas surface surface-floating surface-sunken surface-hover surface-pressed text text-secondary text-help text-placeholder text-disabled border-divider border-control focus link link-hover accent-fill accent-label selected-fill selected-label text-selection-fill text-selection-label overlay-scrim overlay-surface overlay-text info info-fill success success-fill warning warning-fill danger danger-fill destructive-label'.split())
STATE_NAMES = set('default hover pressed focus-visible selected disabled invalid loading'.split())
TEXT_ROLES = set('text text-secondary text-help text-placeholder link link-hover accent-label selected-label text-selection-label overlay-text info success warning danger destructive-label'.split())


def load_usage(path):
    usage = read_json(path)
    if not isinstance(usage, dict) or usage.get("schemaVersion") != 1 or not isinstance(usage.get("roles"), dict) or not isinstance(usage.get("states"), dict):
        raise TokenError("usage: expected schemaVersion 1, roles and states")
    if set(usage["roles"]) != ROLE_NAMES or set(usage["states"]) != STATE_NAMES:
        raise TokenError("usage: retain all approved UI roles and eight interaction states")
    for name, role in usage["roles"].items():
        if not isinstance(role, dict) or not isinstance(role.get("kind"), str) or role.get("kind") not in {"fill", "text", "disabled", "decorative", "border", "focus", "overlay"}:
            raise TokenError(f"usage.{name}: unknown role kind")
        expected = {"text-disabled": "disabled", "border-divider": "decorative", "border-control": "border", "focus": "focus", "overlay-scrim": "overlay"}.get(name, "text" if name in TEXT_ROLES else "fill")
        if role["kind"] != expected:
            raise TokenError(f"usage.{name}: preserve its {expected} role and contrast policy")
        if not isinstance(role.get("token"), str) or "minimumContrast" not in role:
            raise TokenError(f"usage.{name}: define a token reference and explicit contrast threshold or exemption")
        if not isinstance(role.get("description"), str) or not role["description"].strip():
            raise TokenError(f"usage.{name}: describe the role")
        backgrounds = role.get("backgrounds")
        if not isinstance(backgrounds, list) or len(backgrounds) != len(set(map(str, backgrounds))):
            raise TokenError(f"usage.{name}: backgrounds must be a unique list")
        minimum = role.get("minimumContrast")
        floor = 4.5 if role["kind"] == "text" else 3 if role["kind"] in {"border", "focus"} else None
        if floor is not None and (not backgrounds or not number(minimum) or minimum < floor):
            raise TokenError(f"usage.{name}: require backgrounds and minimum contrast {floor}:1")
        if floor is None and minimum is not None:
            raise TokenError(f"usage.{name}: this role has no text contrast exemption or threshold to override")
        if (role["kind"] == "disabled") != (name == "text-disabled"):
            raise TokenError("usage: only text-disabled may use the disabled exemption")
        if name == "text-disabled" and role["token"] != "semantic.color.ink-disabled":
            raise TokenError("usage.text-disabled: bind the unavailable-control foreground")
        if role["token"] == "semantic.color.ink-disabled" and name != "text-disabled":
            raise TokenError(f"usage.{name}: disabled text must not be used for ordinary content or help")
    for name, state in usage["states"].items():
        if not isinstance(state, dict) or "minimumContrast" not in state or any(not isinstance(state.get(key), str) or not state[key].strip()
                                              for key in ("foreground", "background", "appliesTo", "behaviour")):
            raise TokenError(f"usage.states.{name}: define foreground, background, applicability and behaviour")
        minimum = state.get("minimumContrast")
        if name == "disabled":
            if minimum is not None or state["foreground"] != "semantic.color.ink-disabled":
                raise TokenError("usage.states.disabled: reserve disabled text and its explicit exemption for unavailable controls")
        elif not number(minimum) or minimum < 4.5 or state["foreground"] == "semantic.color.ink-disabled":
            raise TokenError(f"usage.states.{name}: active states require readable text at 4.5:1 or above")
    return usage


def usage_pairs(usage):
    for role in usage["roles"].values():
        if role["minimumContrast"] is not None:
            for background in role["backgrounds"]:
                yield role["token"], background, role["minimumContrast"]
    for state in usage["states"].values():
        if state["minimumContrast"] is not None:
            yield state["foreground"], state["background"], state["minimumContrast"]


def validate_usage(usage, composition):
    paths = []
    for role in usage["roles"].values():
        paths.extend([role["token"], *role["backgrounds"]])
    for state in usage["states"].values():
        paths.extend([state["foreground"], state["background"]])
    for path in paths:
        if (not isinstance(path, str) or not re.fullmatch(r"(?:semantic\.color|component\.button\.primary)\.[a-z][a-z0-9-]*", path)
                or path not in composition.tokens or composition.tokens[path]["$type"] != "color"):
            raise TokenError(f"usage: {path!r} must reference an existing semantic or component colour role")
    for name, role in usage["roles"].items():
        if role["kind"] == "fill" and composition.values[role["token"]].get("alpha", 1) != 1:
            raise TokenError(f"usage.{name}: content surfaces must be opaque")
    active = [role["token"] for role in usage["roles"].values() if role["kind"] in {"text", "focus", "border"}]
    active.extend(state["foreground"] for name, state in usage["states"].items() if name != "disabled")
    for original in active:
        path = original
        while path in composition.tokens:
            if path == "semantic.color.ink-disabled":
                raise TokenError(f"usage: {original} must not alias disabled text for active content or help")
            value = composition.tokens[path]["$value"]
            if not isinstance(value, str):
                break
            path = value[1:-1]
