---
id: USER-001
title: Recipes agnósticos de gerenciador de pacotes
scope: recipes
created: 2026-09-04
updated: 2026-09-04
certainty: high
---

# USER-001 — Recipes agnósticos de gerenciador de pacotes

> **TLDR**: recipes deixam de declarar o gerenciador de pacotes — o recipe diz **qual pacote** (opcionalmente com nome por PM) e o qwert resolve internamente qual PM da plataforma usar. `qwert install asdf` vira `brew install asdf` (macOS), `apt-get install asdf` (Debian) ou `pacman -S asdf` (Arch).

> **Fase:** F1  ·  **Tipo:** BE
> **Depende de:** —  ·  **Habilita:** —
> **Telas / rotas:** CLI — `qwert install`, `qwert use`, `qwert apply`

## Contexto

Hoje o recipe declara **como** instalar no `[meta] type = "brew" | "apt" | "pacman" | "qwert"`. Quando o recipe é `brew` numa máquina Linux, o `ensure()` do adapter tenta instalar Homebrew; e quando um recipe não pode ser resolvido (por exemplo um recipe custom `qwert` sem seção para a plataforma Arquitetura), o comando cai no `default_adapter()` e executa `pacman -S <nome>` no Arch — o que produziu o erro real `erro: alvo não encontrado: powerlevel10k`.

O mesmo `config.yml` versionado deve replicar o ambiente em qualquer máquina. A plataforma (e seu PM) é um fato da máquina, não uma escolha do recipe. O recipe só precisa dizer qual pacote quer — com nome por PM, já que o mesmo pacote tem nomes diferentes por gerenciador (ex: `opencode` é `anomalyco/tap/opencode` no brew mas `opencode` no repositório do Arch).

## Design

### Schema do recipe (alvo)

```toml
# install.toml — forma agnóstica (padrão)
[meta]
name = "asdf"
# sem `type` → pacote: o PM da plataforma instala `packages.<pm>` (fallback meta.name)

# nome do pacote por gerenciador (opcional; ausente = meta.name)
packages = { brew = "asdf", apt = "asdf", pacman = "asdf" }

# ferramentas fora de gerenciador de pacotes continuam explícitas:
# [install] macos/debian/arch (a presença de [install] já implica "custom")
```

Regras de resolução:

| Sinal do recipe | Interpretação |
|---|---|
| tem `[install]`/`[upgrade]`/`[uninstall]` com comandos | `custom` — executa os passos da plataforma |
| `packages.<pm_da_plataforma>` presente | usa esse nome no PM da plataforma |
| `packages` ausente ou sem a chave do PM | usa `meta.name` no PM da plataforma |
| legado `type = "brew" \| "apt" \| "pacman"` | deprecado, tratado como pacote no PM da plataforma |
| legado `pkg = "..."` | usado apenas no brew; nas demais plataformas usa `meta.name` |
| legado `type = "qwert"` | deprecado, tratado como `custom` |

### Camada de package management (yuiop)

`yuiop` — assim como `qwert` abre a linha superior do teclado, `yuiop` fecha: juntos cobrem `QWERTYUIOP`. O yuiop é um **binário standalone** (`brew`/`apt`/`pacman` wrapper universal) chamado pelo qwert como subprocesso (`yuiop --json <verb> <canonical>`). O qwert passa apenas o **nome canônico** do recipe; o yuiop detecta a plataforma, resolve o nome por PM (`packages` / catálogo embutido) e executa o gerenciador. Substitui a antiga dupla `for_kind()` + `default_adapter()` (e os adapters `brew.rs`/`apt.rs`/`pacman.rs` embutidos) por um ponto único:

```
recipe → `yuiop <verb> <canonical> --json` → PM da plataforma (brew/apt/pacman)
```

| Plataforma | PM | Instalação (ex.) | Nota |
|---|---|---|---|
| macOS | brew | `brew install <pkg>` | yuiop resolve o nome no tag |
| Debian | apt | `apt-get install -y <pkg>` | yuiop usa sudo |
| Arch | pacman | `pacman -S --noconfirm <pkg>` | yuiop usa sudo |

Efeitos colaterais: `src/adapters/` vira só a ponte do subprocesso (com testes de parsing); os fallbacks de `default_adapter()` somem (só o `yuiop` resolve); `qwert platform <macos|debian|arch>` apenas repassa o override para `yuiop platform <brew|apt|pacman>`, que persiste em `~/.config/yuiop/config.yml`.

## História

Como dev que replica o ambiente em máquinas diferentes, quero declarar um recipe sem amarrar a um gerenciador de pacotes, para o mesmo `config.yml` funcionar em macOS, Debian e Arch sem edição.

## Critérios de aceite

```gherkin
# language: pt
Funcionalidade: Recipes agnósticos de gerenciador de pacotes

  Contexto:
    Dado o qwert instalado em uma máquina com plataforma detectada
    E a ferramenta declarada no config.yml

  Cenário: pacote com tabela por gerenciador instala via PM da plataforma
    Dado uma recita de pacote com packages.brew, packages.apt e packages.pacman
    Quando o usuário roda `qwert install <recita>`
    Então o qwert executa a instalação no PM da plataforma usando o nome da tabela
    E a ferramenta é marcada como instalada no estado

  Cenário: pacote sem tabela usa meta.name no PM da plataforma
    Dado uma recita de pacote sem a tabela packages
    Quando o usuário roda `qwert install <recita>`
    Então o qwert executa a instalação no PM da plataforma usando meta.name

  Cenário: recita legada com type de PM é tratada como pacote
    Dado uma recita com type = "brew", sem pkg e sem seções [install]
    E a plataforma atual não é macOS
    Quando o usuário roda `qwert install <recita>`
    Então o qwert trata a recita como pacote e instala via PM da plataforma usando meta.name

  Cenário: recita custom executa os passos explícitos da plataforma
    Dado uma recita com seções [install] para a plataforma atual
    Quando o usuário roda `qwert install <recita>`
    Então o qwert executa os comandos explícitos da seção
    E não consulta o gerenciador de pacotes

  Cenário: recita custom sem cobertura da plataforma falha com mensagem clara
    Dado uma recita custom com [install] apenas para macOS
    E a plataforma atual é Arch
    Quando o usuário roda `qwert install <recita>`
    Então a instalação falha com mensagem informando que a recita não cobre a plataforma
    E o qwert não executa o PM da plataforma com o nome genérico da ferramenta

  Cenário: pkg legado específico de brew é ignorado fora do brew
    Dado uma recita com pkg = "anomalyco/tap/opencode"
    E a plataforma atual é Arch
    Quando o usuário roda `qwert install opencode`
    Então o pkg legado é ignorado por ser específico de brew
    E a instalação usa meta.name ("opencode") no PM da plataforma

  Cenário: plataforma não detectada orienta a definir explicitamente
    Dado uma máquina sem apt e sem pacman em /usr/bin
    E nenhuma plataforma override definida
    Quando o usuário roda `qwert apply` ou `qwert install`
    Então o qwert não tenta instalar um PM estranho à plataforma
    E informa que é preciso rodar `qwert platform <macos|debian|arch>`

  Cenário: plataforma definida explicitamente é respeitada
    Dado que o usuário rodou `qwert platform arch` em um servidor
    Quando o usuário roda `qwert install <recita-de-pacote>`
    Então a recita resolve para o pacman mesmo que a auto-detecção falhe
    E o Homebrew não é instalado no Linux

  Cenário: Homebrew nunca é instalado automaticamente fora do macOS
    Dado um recipe que por engano resolveria para brew em Linux
    Quando o qwert tenta garantir o brew
    Então a instalação falha com mensagem explicando que Homebrew só é instalado no macOS
    E não executa o instalador do Homebrew
```

## Non-goals

- Não expor comando de usuário `yuiop` **no qwert** — a superfície continua `qwert install/use/apply`; o `yuiop` é um binário separado que o qwert invoca.
- Não adicionar PMs além dos nativos (brew/apt/pacman) nesta fase; a tabela `packages` já permite estender depois (ex: `aur`).
- Não migrar os recipes existentes de volta — compatibilidade preservada via regras de deprecação.
- Não mudar o mecanismo de distribuição de recipes (ver spec de split/plugins abaixo).
- A auto-instalação do Homebrew continua sendo recurso **exclusivo do macOS**; em Linux o usuário usa o PM nativo ou define a plataforma via `qwert platform <platform>`.

## Dependências & referências

- Spec relacionada: `.project/docs/specs/20260904163512_split_recipes_plugin_system.md` — o schema de `install.toml` documentado no README do futuro repo `qwert-recipes` deve incluir a tabela `packages` e os tipos novos; e a resolução por PM deve valer igualmente para recipes de plugins.
- Código afetado: `src/adapters/` (`mod.rs`, `yuiop.rs`), `src/recipe/runner.rs`, `src/recipe/index.rs` (`default_kind`), `src/commands/apply.rs`, `use_cmd.rs`, `install_cmd.rs`, `uninstall_cmd.rs`, `drop_cmd.rs`, `src/config/qwert_yml.rs` (`packages` inline se aplicável).
- Regras: `.project/ai/rules/recipes.md` — mudança de schema exige bump de `recipes/VERSION`.