---
title: Roles e multi-máquina
spec: null
created: 2026-09-03
updated: 2026-09-03
certainty: high
---

# Roles e multi-máquina — Plano de implementação

> **TLDR**: transformar o `config.yml` de uma lista flat de tools para seções por role (`shared` implícito + roles como `dev`, `server`, `macos`, ...). A identidade da máquina (`~/.local/share/qwert/machine.yml` + env `QWERT_ROLES`) decide quais seções se aplicam. Adicionar overrides de dotfiles por role (`~/.qwert/<tool>/overrides/<role>/` → merged dir) e suporte a `arch` nos recipes.

> **Spec:** sem spec — decisões tomadas via grilling (entrevista)
> **Branch:** `main`

**Arquitetura:** o conceito é **role** (não profile). Roles são seções nomeadas pelo dev no `config.yml`; `shared` é um role especial implícito sempre ativo. A máquina declara seus roles na instalação (prompt) ou via `qwert machine` / `QWERT_ROLES`. Uma tool é instalada se aparece em qualquer seção ativa (união); o "último vence" (ordem dos roles no machine.yml) decide version/setup para tools duplicadas.

**Stack:** Rust (clap, serde_yml, indexmap).

## Restrições globais

- Backward compat: `tools:` flat atual e `hooks:` flat atual são lidos como seção/role `shared` implícito — instalações existentes não quebram.
- `macos`/`linux` são roles normais (o dev nomeia como quiser). Sem auto-detect de plataforma para seleção.
- Ordem dos roles no `machine.yml` = precedência (último vence). `shared` é sempre a base da pilha.
- Merge materializado em `~/.local/share/qwert/merged/<tool>/`, recriado a cada `apply`.
- `arch` como seção nos recipes/inline setup, com fallback para `debian` no Arch (preserva comportamento atual).

---

### Task 1: Refatorar `src/config/qwert_yml.rs` — seções + API role-aware

**Files:**
- Modify: `src/config/qwert_yml.rs`

**Interfaces:**
- Consumes: schema atual (ToolEntry, InlineSetup)
- Produces: `QwertConfig.tools: IndexMap<String, IndexMap<String, ToolEntry>>`, `hooks: IndexMap<String, RoleHooks>`, constantes `SHARED`

- [x] **Step 1: Deserialização customizada**
  - `tools:`: se todos os valores são "seções" (mappings cujas chaves não são `version`/`setup`) → seções; senão flat → wrapper em `shared`.
  - `hooks:`: valores arrays → flat (wrapper em `shared`); valores mappings → seções.
- [x] **Step 2: API role-aware**
  - `effective_sections(&roles) -> Vec<String>` — `[shared] + roles` em ordem (dedup).
  - `tool_names_for_roles(&roles) -> Vec<String>` — união em ordem de seção.
  - `version_of_for_roles(name, &roles)`, `setup_of_for_roles(name, &roles)` — último vence.
  - `role_sections()`, `ensure_section(role)`, `declared_anywhere(name)`, `has_tool_in(role, name)`.
  - `add_tool(name, role, version)`, `remove_tool(name)` (remove de todas as seções, limpa seções vazias), `add_hook(role, hook, path)`.
- [x] **Step 3: Campo `arch`** em `InlineSetup` e `InlineUndo`.

### Task 2: Criar `src/config/machine.rs` — identidade da máquina

**Files:**
- Create: `src/config/machine.rs`
- Modify: `src/config/mod.rs`

- [x] **Step 1: `MachineIdentity { roles: Vec<String> }`**
  - `load()`: `QWERT_ROLES` (csv) vence o arquivo; senão lê `machine_path()`.
  - `save()`, `set_roles()`.
- [x] **Step 2: `machine_path()`** → `~/.local/share/qwert/machine.yml`.

### Task 3: Criar `src/config/merge.rs` — materialização de overrides

**Files:**
- Create: `src/config/merge.rs`
- Modify: `src/platform/fs.rs` (helper `copy_dir`, `copy_dir_excluding`)

- [x] **Step 1: `materialize(tool, roles, config_dir, data_dir) -> Result<Option<PathBuf>>`**
  - Se nenhum `overrides/<role>` existe → `None` (usa `~/.qwert/<tool>/` direto).
  - Senão: recria `merged/<tool>/`, copia base (exceto `overrides/`), aplica cada role em ordem (sobrescreve). Retorna `Some(merged)`.

### Task 4: Campo `arch` nos recipes

**Files:**
- Modify: `src/recipe/schema.rs`, `src/recipe/index.rs`, `src/recipe/runner.rs`

- [x] **Step 1:** adicionar `arch: Option<Commands>` em `RecipeInstall`, `RecipeUpgrade`, `RecipeUninstall`, `RecipeSetup`, `SetupUndo`, `InstallFile`, `SetupFile`.
- [x] **Step 2:** `platform_cmds` — `Arch => arch.or(debian)` (fallback para debian).
- [x] **Step 3:** mapear `arch` em `index::assemble_recipe` e `runner::setup_inline`.

### Task 5: CLI — comando `machine` + flags `--role`

**Files:**
- Modify: `src/cli.rs`, `src/main.rs`, `src/commands/mod.rs`
- Create: `src/commands/machine_cmd.rs`

- [x] **Step 1:** `Command::Machine { roles: Vec<String> }`.
- [x] **Step 2:** `--role` em `Use::Tool` (parse manual no main.rs) e `Use::Script`.
- [x] **Step 3:** `machine_cmd::run(roles)` — sem args imprime; com args salva, valida seções do config (avisa seção vazia), sugere `qwert apply`.

### Task 6: Comandos role-aware

**Files:**
- Modify: `src/commands/apply.rs`, `use_cmd.rs`, `install_cmd.rs`, `setup_cmd.rs`, `uninstall_cmd.rs`, `drop_cmd.rs`, `status.rs`, `list.rs`, `upgrade.rs`, `info.rs`, `doctor.rs`, `hook.rs`

- [x] **Step 1:** `apply.rs` — `effective_sections` + `tool_names_for_roles`; orphans contra tools ativas; materializa merge e usa como `from` do setup.
- [x] **Step 2:** `use_cmd.rs` — `use_tool(name, version, role, no_install)`, `use_script(hook, path, role)`.
- [x] **Step 3:** `install/setup` — declaram em `shared`; `setup` usa `setup_of_for_roles`.
- [x] **Step 4:** `uninstall/drop` — `declared_anywhere` + `remove_tool` (todas as seções).
- [x] **Step 5:** `status/list/upgrade/doctor` — usam tools ativas da máquina.
- [x] **Step 6:** `hook.rs` — merge de hooks por role (shared + roles em ordem).
- [x] **Step 7:** `self_cmd::install()` — prompt interativo de roles se `machine.yml` não existe.

### Task 7: Testes

**Files:**
- Create: `src/config/tests/machine.rs`, `src/config/tests/merge.rs`
- Modify: testes de `qwert_yml.rs` (API nova), `schema.rs` (arch)

- [x] **Step 1:** testes de deserialização V1/V2, união, último vence, add/remove por role.
- [x] **Step 2:** testes de load/save/env override do machine.
- [x] **Step 3:** testes de merge (sem overrides, com overrides em ordem, base + overrides).
- [x] **Step 4:** testes de parsing `arch` em recipes e inline setup.

### Task 8: Validar

- [x] **Step 1:** `make t`
- [x] **Step 2:** `make build`
- [x] **Step 3:** `cargo clippy` (se disponível)