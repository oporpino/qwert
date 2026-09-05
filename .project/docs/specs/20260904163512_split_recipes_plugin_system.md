---
title: Remover recipes do qwert e criar sistema de plugins
status: in_progress
created: 2026-09-04
updated: 2026-09-04
owner: "@oporpino"
certainty: high
---

# Remover recipes do qwert e criar sistema de plugins

> **TLDR**: `recipes/` sai do repo qwert para um repo próprio `qwert-recipes`, sempre sintetizado via git clone. Um novo comando `qwert plugin add <url>` (estilo asdf) permite que devs apontem repos de recipes extras, declarados no `~/.qwert/config.yml`.

## Contexto

Os recipes de hoje vivem dentro do repo qwert (`recipes/<name>/`) e são distribuídos por tarball com um mecanismo de versionamento por `recipes/VERSION` + bump automático em CI. Isso acopla o ciclo de vida de recipes (comunidade) ao do binário e dificulta que terceiros compartilhem recipes.

O objetivo é separar o catálogo de recipes do código do qwert e permitir que qualquer dev publique seu próprio repo de recipes como "plugin", análogo ao `asdf plugin add`.

## Objetivos

- Remover `recipes/` do repo qwert e publicar em repo próprio `br4zz4/qwert-recipes`.
- Substituir o mecanismo tarball+VERSION por **git clone/pull** tanto para o catálogo default quanto para os plugins.
- Criar comando declarativo `qwert plugin add <url>` (name derivado do URL) que registra o plugin no `~/.qwert/config.yml` (versionável e replicável).
- Criar README no `qwert-recipes` que ensina devs a criar seus próprios repos de recipes.
- Manter precedência de busca: recipes locais (`~/.qwert/recipes`) > plugins (ordem de declaração) > catálogo default.

## Fora de escopo

- Registry público de plugins pesquisável (ex: `search` em plugins de terceiros).
- Template GitHub de repo de recipes (o README do `qwert-recipes` serve de guia e modelo).
- Migração automática de usuários existentes — no próximo `qwert recipes update` o cache é substituído pelo git clone.
- Alterações no fluxo de release/install do binário (org de releases, `scripts/install.sh`, `self upgrade`).

## Mudanças

### 1. Novo repo `br4zz4/qwert-recipes`

Criado via `gh repo create`. Conteúdo:

- `recipes/<name>/install.toml|setup.toml` — os 13 recipes migrados de `recipes/` (asdf, claude, codex, delta, iterm2, kimi, lvim, neovim, ohmyzsh, openclaw, opencode, powerlevel10k, tmux).
- `README.md` — guia para devs criarem repos de recipes:
  - layout esperado (`recipes/<name>/` com `install.toml` e/ou `setup.toml`)
  - schema de `install.toml` (`[meta]`, `[check]`, `[install]`) e `setup.toml` (symlink, copy, commands, `[undo]`)
  - tipos `brew | apt | pacman | qwert`
  - como apontar o repo via `qwert plugin add <url>`
  - como testar localmente.

### 2. Registro de plugins no config.yml

`~/.qwert/config.yml` (schema):

```yaml
plugins:
  - name: qwert-recipes-neovim
    url: https://github.com/br4zz4/qwert-recipes-neovim
```

- `name` — derivado do URL (último segmento do path, sem `.git`, sanitizado para caracteres alfanuméricos + `-`). Colisão de nome entre dois plugins falha no `add`.
- `url` — URL git do repo.

Mudanças em `src/config/qwert_yml.rs`:
- Novo campo `plugins: Vec<PluginSource>` em `QwertConfigRaw`/`QwertConfig` (serde, default vazio).
- Métodos: `plugins()`, `add_plugin(name, url)`, `remove_plugin(name)`.

### 3. Novo módulo de plugins

`src/plugins.rs` (novo):

- `plugins_dir()` → `~/.local/share/qwert/plugins/`.
- `derive_name(url) -> Result<String>` — extrai/sanitiza o nome.
- `add(url)` — grava no config.yml e faz `git clone <url> <plugins_dir>/<name>`.
- `remove(name)` — remove do config.yml e apaga o clone.
- `list()` → `Vec<(name, url)>` (declarados, indicando clonado ou não).
- `ensure_clones()` — para cada plugin declarado sem clone, clona (usado no `apply` p/ réplica do ambiente).
- `update_all()` — `git pull --ff-only` em todos os clones.
- `dirs()` → paths de busca dos clones na ordem de declaração (usado pelo index).

Convenção de teste: `src/tests/plugins.rs` via `#[path]`, padrão triple-A.

### 4. Mecanismo de atualização default (git clone)

`src/commands/recipes_cmd.rs` — reescrever:

- Remover `TARBALL_URL`, `VERSION_URL`, `fetch_version`, `copy_dir`.
- `update()` — garante/sincroniza o clone default:
  - ausente → `git clone https://github.com/br4zz4/qwert-recipes <data>/recipes`
  - presente → `git pull --ff-only`
- `update_silent()` — `git pull --ff-only --quiet` no clone default, ignorando erros (offline não quebra comandos).
- O clone default fica em `~/.local/share/qwert/recipes/` (path mantido).

### 5. Busca com precedência

`src/recipe/index.rs`:

- `find(name, recipes_dir)` continua recebendo apenas o default; internamente consome `plugins::dirs()`:
  1. `~/.qwert/recipes/<name>` (local override — inalterado)
  2. `~/.local/share/qwert/plugins/<plugin>/recipes/<name>` na ordem de declaração
  3. `<recipes_dir>/<name>` (default)
- `load_all` — inclui recipes de todos os plugins + default, dedup por nome com a precedência vencendo; manter ordenação por nome.
- Testes: precedência local > plugin > default, ordem entre plugins, dedup em `load_all`.

### 6. Novos comandos CLI

`src/cli.rs` + `src/main.rs`:

```
qwert plugin add <url>      # clona + registra no config.yml
qwert plugin remove <name>   # remove do config.yml + apaga clone
qwert plugin list            # lista plugins (nome, url, clonado?)
qwert plugin update          # git pull em todos os plugins
```

- `qwert recipes update` continua existindo (atualiza o default via git).
- `help.rs` — adicionar seção de plugins.

### 7. Integração nos comandos existentes

- `apply.rs`, `use_cmd.rs`, `install_cmd.rs`, `setup_cmd.rs` (que hoje chamam `update_silent`) também chamam `plugins::ensure_clones()` antes de buscar recipes — assim `apply` numa máquina nova restaura os plugins declarados no config.yml versionado.
- `doctor.rs` — checa que os plugins declarados estão clonados.
- `search.rs`/`search_complete_cmd.rs`/`list.rs`/`status.rs`/`info.rs`/`upgrade.rs`/`reinstall.rs`/`versions_cmd.rs` — sem mudança de lógica (usam `index::find`/`load_all`, que passam a incluir plugins automaticamente).

### 8. Limpeza no repo qwert

- Deletar `recipes/` (incluindo `recipes/VERSION`).
- Deletar `.github/workflows/bump-recipes-version.yml`.
- Atualizar `.project/ai/rules/recipes.md` — remover regras de VERSION/tarball, descrever git clone + plugins.
- Atualizar `CLAUDE.md` — arquitetura (recipes agora via git clone + plugins).
- Atualizar `README.md` — documentar `qwert plugin add`.
- `Cargo.toml` — sem novas dependências (git já é invocado via `Command`; qualquer lib externa desnecessária).

## Como verificar

1. `make t` na raiz do qwert — testes passam (incluindo novos de precedência e plugins).
2. `make build`.
3. Manual:
   - `qwert recipes update` — clona o default e popula `~/.local/share/qwert/recipes/`.
   - `qwert search tmux` encontra recipes do default.
   - `qwert plugin add file:///tmp/qwert-recipes-teste` num repo local de teste — registra em `config.yml`, clona em plugins dir, e a recipe do repo de teste aparece em `qwert search`.
   - `qwert plugin list` mostra o plugin; `qwert plugin update` faz pull; `qwert plugin remove` remove do config e apaga o clone.
   - `qwert apply` numa máquina com config.yml + plugin declarado clona o plugin automaticamente.
4. `qwert-recipes`: `git clone` funcionando, recipes disponíveis via `qwert` após `recipes update`.
5. Repo `br4zz4/qwert-recipes` existe com os 13 recipes e README.

## Documentação

- `README.md` do novo repo `br4zz4/qwert-recipes` (guia para devs).
- `CLAUDE.md`, `README.md`, `.project/ai/rules/recipes.md` do qwert atualizados.
- Sem mudanças em `.project/docs/features/` (comportamento interno do dev tool, não produto).