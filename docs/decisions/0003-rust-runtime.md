# ADR 0003: Rust runtime for the foundation CLI

Status: accepted

Date: 12 November 2026

Authority: the project owner approved replacing the Python runtime with a Rust rewrite designed from the contract, not a port.

Amends the Python-runtime wording of [ADR 0002](0002-shared-design-foundation.md). Its scope is unchanged: the CLI still provides only `validate`, `build` and `export`.

## Decision

`fjord` is a Rust crate (`Cargo.toml`, `Cargo.lock`). The Python package, pytest, `pyproject.toml` and `uv.lock` are removed. `scripts/inspect_fonts.py` remains the maintainer Fontconfig probe. Before cutover the Python CLI served once as a differential oracle: 60 positive compositions produced identical resolved semantics, 15 negative overrides were rejected by both CLIs with the expected rule, and both `validate` runs passed.

## Decisions on inherited behaviour

| # | Decision |
| --- | --- |
| L3 | Keep the 300 ms transition limit; restored in the Motion guideline. |
| L4 | Keep licence text markers and the WOFF2 header check. A refreshed hash must not hide a wrong file. |
| L5 | Keep the five-family font baseline with fixed styles. |
| L7-L9 | Drop diff markers, OKLCH gamut mapping and ANSI binding order. Their consumers were removed in the scope cut. |
| L10 | Default output names are `dist/fjord.<mode>.<suffix>`; the consumer guide is corrected. |
| L11 | The default root is found from the current directory (nearest ancestor with `tokens/contract.json`) or `--root`. |
| L12 | Property-level pointers do not supply a token type. |
| L13 | Alias chains have no length limit. |
| L14 | CSS numbers are canonical (`15px`); single-string `fontFamily` is one family; all CSS generic keywords stay unquoted. |
| L15 | All independent violations are reported, each with a stable `[rule-id]`. |
| L16 | Inside the project root, output is written only under `dist/`, never over a supplied layer file. |
| L17 | `validate` covers every official theme for every mode. |

## Observable contract

Unchanged: commands, flags, exit codes 0/1/2, `PASS:`/`Wrote`/`WARN`/`ERROR` line prefixes, resolved JSON schema and `source` provenance grammar, DTCG export content, CSS names and value semantics. Incidental: diagnostic wording, JSON key order and number text.
