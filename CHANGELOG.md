# Changelog

## Unreleased: design foundation only

This is a breaking reduction of the project's delivery scope. The existing token names, values, schema `2`, contract `0.3.0` and `base → mode → theme → overrides` composition are retained.

- Removed component libraries, platform adapters and their application dependencies from the active project.
- Removed previews, Storybook, catalogues, native packages, release archives and their commands.
- Limited the CLI to `validate`, `build` and `export`, with `dtcg`, `resolved` and `css` export formats. Native WezTerm export and `theme-css` are removed.
- Kept the shared design rules, font/icon/brand assets, theme examples, portable exports and validation.

Consumers using removed commands or packages must keep the previous source revision while extracting their implementation into a separate project. New consumers use the [consumer guide](docs/consumer-guide.md).

## Historical combined releases

The owner accepted the combined `1.0.0` release for local distribution on 8 November 2026, and the earlier MVP `0.1.0` on 29 October 2026. Their acceptance and recorded verification limits are unchanged. Their original [changelog](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/CHANGELOG.md), [source](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb) and [verification records](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/verification) remain in Git history. This unreleased scope change does not replace those release artifacts or claim a new acceptance.
