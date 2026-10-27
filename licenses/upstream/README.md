# Upstream licence notices

The Bun package inventory in the local candidate records packages that do not include a root licence file. These original files were retrieved from the source repository at the dependency release tag or npm publication commit on 27 October 2026:

| File | Source |
| --- | --- |
| `storybook-10.6.3.txt` | [Storybook v10.6.3](https://github.com/storybookjs/storybook/blob/v10.6.3/LICENSE) |
| `esbuild-0.28.2.txt` | [esbuild v0.28.2](https://github.com/evanw/esbuild/blob/v0.28.2/LICENSE.md) |
| `oxc-resolver-11.21.2.txt` | [oxc-resolver v11.21.2](https://github.com/oxc-project/oxc-resolver/blob/v11.21.2/LICENSE) |
| `rolldown-1.2.15.txt` | [Rolldown v1.2.15](https://github.com/rolldown/rolldown/blob/v1.2.15/LICENSE) |
| `oxc-0.127.0.txt` | [Oxc crates_v0.127.0](https://github.com/oxc-project/oxc/blob/crates_v0.127.0/LICENSE) |
| `oxc-0.127.0-third-party.txt` | [Oxc third-party notices](https://github.com/oxc-project/oxc/blob/crates_v0.127.0/THIRD-PARTY-LICENSE) |
| `is-reference-3.0.3-README.md` | [is-reference publication commit](https://github.com/Rich-Harris/is-reference/blob/8bb053129bfabe2f6a7d7ed050159d67ebe82829/README.md) |
| `locate-character-3.0.0-README.md` | [locate-character publication commit](https://github.com/Rich-Harris/locate-character/blob/4f08a59ec248121f7002abd02ee7b94e8eda06bc/README.md) |

The Storybook notice is retained for the Storybook packages at 10.6.3. The esbuild, resolver and Rolldown source notices accompany their listed platform binding packages. The Oxc source tag's `napi/parser/package.json` confirms parser version 0.127.0; its own licence and third-party notices are retained together.

The npm publication commits for is-reference and locate-character contain an MIT declaration in README and package metadata, but no separate licence file or copyright notice. Those declarations are preserved as supplied. No copyright attribution or licence text has been invented to fill that upstream gap.

The inventory includes build tools as well as code in the catalogue. Complete the review of the code actually distributed in Storybook before accepting the release.
