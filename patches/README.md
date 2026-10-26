# Dependency patches

`@storybook/react@10.6.3` has a declaration-only patch at `@storybook%2Freact@10.6.3.patch`. Bun applies it during installation through the root manifest's `patchedDependencies` entry. Version the patch together with the manifest and lockfile. See [Bun's patch documentation](https://bun.sh/docs/pm/cli/patch).

## Storybook factory declarations

Storybook's shared declarations reference its React renderer even in the Svelte catalogue. The published factory types produce four diagnostics during strict checking with both TypeScript 6.0.3 and 7.0.2.

`ReactMeta` supplies its own input and story overloads. Extending the generic base with that specialized input creates a constraint mismatch. The patch keeps the base's remaining members through `Omit`, then declares the existing input and overloads explicitly.

Mocked callbacks also change the argument type of a factory story. `ReactStory` now accepts that argument type as a defaulted third parameter and checks its input against it. The return type passes the mocked argument type through. The existing two-parameter use, component props and other story members retain their types. The two obsolete inline error directives are removed.

The patch changes one `.d.ts` file. It adds no `any`, casts, error directives, diagnostic filters or compiler exclusions. Runtime JavaScript and dependency versions are unchanged. The [strict regression fixtures](../tests/types/storybook/valid.ts) exercise defaults, required args, callback mock signatures, typed results and render overloads. Five separate invalid fixtures must fail with their expected diagnostics, including Svelte story args.

Run from the repository root:

```sh
mise run test:storybook-types
mise run ci
mise run test:clean
```

When a newer stable Storybook release fixes these declarations, remove the `patchedDependencies` entry and patch, update the exact versions and lockfile, then rerun the same checks. Do not carry the patch to another version without reviewing its declarations.

The modified declarations originate in [Storybook](https://github.com/storybookjs/storybook), copyright 2024 Storybook, under MIT. The upstream [licence text](LICENSE.storybook) is retained alongside the patch.
