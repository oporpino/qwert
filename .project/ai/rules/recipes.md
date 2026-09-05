## recipes

- recipes live in `recipes/<name>/` inside recipe repos (the default `br4zz4/qwert-recipes` or user plugins) with `install.toml` and/or `setup.toml`
- schema fields use `from` (not `src`), `to`, `symlink`, `macos`, `debian`, `undo`
- `load_toml_opt` silently returns `None` on parse failure — a wrong field name means setup is silently ignored

## Catalog delivery

The default catalog and all plugins are delivered via **git clone** (no tarballs, no VERSION file).

- default → cloned to `~/.local/share/qwert/recipes/` from `https://github.com/br4zz4/qwert-recipes`
- plugins → cloned to `~/.local/share/qwert/plugins/<name>/`, declared in `~/.qwert/config.yml`

`update_silent()` in `src/commands/recipes_cmd.rs` pulls the default catalog
silently (ignoring errors for offline use) and `ensure_clones()` clones any declared
plugins — both run before recipe lookup in `use`/`install`/`setup`/`apply`.

Recipe lookup precedence: `~/.qwert/recipes` (local override) → plugin repos (declaration
order) → default catalog.

### Rules

- never put recipes in this repo — they live in `br4zz4/qwert-recipes` or user plugins
- when editing the catalog, push directly to `br4zz4/qwert-recipes` (git pull ships it)
- when adding or renaming fields in `RecipeSetup` or `SetupFile`, remember silent parse failures are hard to debug