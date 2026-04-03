## recipes

- recipes live in `recipes/<name>/` with `install.toml` and/or `setup.toml`
- schema fields use `from` (not `src`), `to`, `symlink`, `macos`, `debian`, `undo`
- `load_toml_opt` silently returns `None` on parse failure — a wrong field name means setup is silently ignored

## VERSION

`recipes/VERSION` is the single source of truth for the recipes cache version.
`update_silent()` in `src/commands/recipes_cmd.rs` fetches only this file first;
if it matches the local cache, no tarball download happens.

**A stale VERSION = users never pick up recipe changes.**

### In CI (normal flow)

`.github/workflows/bump-recipes-version.yml` auto-bumps `recipes/VERSION` on every
push to `main` that touches `recipes/**` (excluding `recipes/VERSION` itself).
No manual action needed.

### Locally (bypassing CI)

If you change any recipe file outside of a CI push (local testing, direct edits), bump the version manually:

```bash
date -u +%Y%m%d%H%M%S > recipes/VERSION
```

Commit it together with the recipe change — never in a separate commit.

### Rules

- never modify a recipe file without also updating `recipes/VERSION`
- never update `recipes/VERSION` without a corresponding recipe change
- when adding or renaming fields in `RecipeSetup` or `SetupFile`, bump VERSION immediately — silent parse failures are hard to debug
