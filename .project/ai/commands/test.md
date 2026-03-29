# Test

Create tests for files changed in the current diff.

## Test file structure

This project keeps test files **separate** from source files using Rust's `#[path]` attribute.

```
src/
  adapters/
    brew.rs              ← source file
    tests/
      brew.rs            ← test file
```

### Linking in the source file

```rust
// src/adapters/brew.rs

#[cfg(test)]
#[path = "tests/brew.rs"]
mod tests;
```

### Imports in the test file

```rust
// src/adapters/tests/brew.rs

use super::super::BrewAdapter;  // super = brew module, super::super = adapters
```

### Module-level tests (mod.rs)

```rust
// src/adapters/mod.rs

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
```

## Steps

1. Identify changed files: `git diff --name-only HEAD`
2. For each changed source file, create `src/<module>/tests/<name>.rs`
3. Add `#[cfg(test)] #[path = "tests/<name>.rs"] mod tests;` to the source file if not present
4. Write tests following `@../../../commons/ai/rules/shared/testing.md`:
   - Triple A: `// arrange`, `// act`, `// assert`
   - Atomic — everything inside the test
   - One assert per test (max 3)
5. Run `make t` — if a test fails, report to the user; never change production code to fix a test

## Rules

- Never modify existing test files without user confirmation
- Never change production code to make a test pass
