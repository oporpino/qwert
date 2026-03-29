#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="oporpino"
REPO_NAME="qwert"
REPO="https://github.com/${REPO_OWNER}/${REPO_NAME}"
API="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"

QWERT_HOME="${HOME}/.qwert"
QWERT_BIN="${QWERT_HOME}/bin"
QWERT_RECIPES="${QWERT_HOME}/recipes"
QWERT_VERSION="${QWERT_VERSION:-}"  # optional: pin a specific tag (e.g. v0.2.1)

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

info()    { printf "  \033[1m→\033[0m  %s\n" "$*"; }
ok()      { printf "  \033[32m✓\033[0m  %s\n" "$*"; }
warn()    { printf "  \033[33m!\033[0m  %s\n" "$*"; }
die()     { printf "  \033[31m✗\033[0m  %s\n" "$*" >&2; exit 1; }
dim()     { printf "  \033[2m%s\033[0m\n" "$*"; }

need() {
    command -v "$1" &>/dev/null || die "required: $1 not found"
}

# ---------------------------------------------------------------------------
# platform
# ---------------------------------------------------------------------------

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}" in
        Darwin)
            case "${arch}" in
                arm64)  echo "aarch64-apple-darwin" ;;
                x86_64) echo "x86_64-apple-darwin" ;;
                *)      die "unsupported macOS arch: ${arch}" ;;
            esac
            ;;
        Linux)
            case "${arch}" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                *)        die "unsupported Linux arch: ${arch}" ;;
            esac
            ;;
        *)
            die "unsupported OS: ${os}"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# ask config dir
# ---------------------------------------------------------------------------

ask_config_dir() {
    local default="${HOME}/.config"
    local input

    # If ~/.qwert/config already points to a dir with qwert.yml, use it as-is
    local persisted=""
    if [ -f "${QWERT_HOME}/config" ]; then
        persisted="$(grep '^QWERT_CONFIG_DIR=' "${QWERT_HOME}/config" | cut -d= -f2)"
        persisted="${persisted/#\~/${HOME}}"
    fi

    local candidate="${persisted:-${default}}"
    if [ -f "${candidate}/qwert.yml" ]; then
        QWERT_CONFIG_DIR="${candidate}"
        ok "Config dir: ${QWERT_CONFIG_DIR}  (existing qwert.yml found)"
        return
    fi

    printf "\n"
    printf "  \033[1mConfig directory\033[0m\n"
    dim "This is where qwert.yml will be created (e.g. ~/.config/qwert.yml)."
    dim "Save this folder in a personal git repo to replicate your environment on any machine."
    printf "\n"
    printf "  Location [~/.config]: "
    read -r input

    if [ -z "${input}" ]; then
        QWERT_CONFIG_DIR="${default}"
    else
        QWERT_CONFIG_DIR="${input/#\~/${HOME}}"
    fi

    mkdir -p "${QWERT_CONFIG_DIR}"

    # Persist non-default config dir to ~/.qwert/config (read by qwert at runtime)
    if [ "${QWERT_CONFIG_DIR}" != "${default}" ]; then
        mkdir -p "${QWERT_HOME}"
        printf 'QWERT_CONFIG_DIR=%s\n' "${QWERT_CONFIG_DIR}" > "${QWERT_HOME}/config"
    fi

    ok "Config dir: ${QWERT_CONFIG_DIR}"

    # Suggest git init if not already a repo
    if [ ! -d "${QWERT_CONFIG_DIR}/.git" ]; then
        printf "\n"
        warn "This folder is not a git repository."
        printf "  Initialize it now? [Y/n]: "
        read -r git_choice
        if [[ -z "${git_choice}" || "${git_choice}" =~ ^[Yy]$ ]]; then
            git -C "${QWERT_CONFIG_DIR}" init -q
            ok "Git repo initialized at ${QWERT_CONFIG_DIR}"
            dim "Tip: push it to GitHub and run qwert apply on any new machine."
        fi
    fi
}

# ---------------------------------------------------------------------------
# install binary
# ---------------------------------------------------------------------------

install_binary() {
    local target version download_url tmp

    target="$(detect_target)"
    info "Platform: ${target}"

    if [ -n "${QWERT_VERSION}" ]; then
        version="${QWERT_VERSION}"
        info "Version: ${version} (pinned)"
    else
        info "Fetching latest release..."
        version="$(curl -fsSL "${API}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"
        [ -n "${version}" ] || die "could not determine latest release version"
        info "Version: ${version}"
    fi

    download_url="${REPO}/releases/download/${version}/qwert-${target}"
    tmp="$(mktemp)"

    info "Downloading binary..."
    curl -fsSL --progress-bar "${download_url}" -o "${tmp}" \
        || die "download failed: ${download_url}"

    mkdir -p "${QWERT_BIN}"
    mv "${tmp}" "${QWERT_BIN}/qwert"
    chmod +x "${QWERT_BIN}/qwert"

    echo "${version}" > "${QWERT_HOME}/version"
    ok "Binary installed → ${QWERT_BIN}/qwert (${version})"
}

# ---------------------------------------------------------------------------
# install from source (fallback)
# ---------------------------------------------------------------------------

install_from_source() {
    info "No release binary found — building from source..."
    need cargo

    local tmp_dir
    tmp_dir="$(mktemp -d)"

    info "Cloning repository..."
    git clone --depth 1 "${REPO}.git" "${tmp_dir}" \
        || die "git clone failed"

    info "Building (release)..."
    (cd "${tmp_dir}" && cargo build --release --quiet) \
        || die "cargo build failed"

    mkdir -p "${QWERT_BIN}"
    mv "${tmp_dir}/target/release/qwert" "${QWERT_BIN}/qwert"
    chmod +x "${QWERT_BIN}/qwert"
    rm -rf "${tmp_dir}"
    ok "Binary built and installed → ${QWERT_BIN}/qwert"
}

# ---------------------------------------------------------------------------
# install completions
# ---------------------------------------------------------------------------

install_completions() {
    local shell_name
    shell_name="$(basename "${SHELL:-bash}")"

    if [ "${shell_name}" = "zsh" ]; then
        local comp_dir="${QWERT_HOME}/completions"
        mkdir -p "${comp_dir}"
        "${QWERT_BIN}/qwert" completions zsh > "${comp_dir}/_qwert"
        ok "Zsh completions installed → ${comp_dir}/_qwert"
    fi
}

# ---------------------------------------------------------------------------
# install recipes
# ---------------------------------------------------------------------------

install_recipes() {
    info "Downloading recipes..."
    "${QWERT_BIN}/qwert" recipes update \
        || die "failed to download recipes"
}

# ---------------------------------------------------------------------------
# configure shell
# ---------------------------------------------------------------------------

configure_shell() {
    local rc_file

    if [ -f "${HOME}/.zshrc" ]; then
        rc_file="${HOME}/.zshrc"
    elif [ -f "${HOME}/.bashrc" ]; then
        rc_file="${HOME}/.bashrc"
    else
        warn "no .zshrc or .bashrc found — configure PATH and QWERT_CONFIG_DIR manually"
        return
    fi

    # Remove any existing qwert lines before reinstalling
    grep -vE '(# qwert|\.qwert/bin|qwert hook|qwert completions|source <\(qwert|\.qwert/completions)' \
        "${rc_file}" > "${rc_file}.tmp" && mv "${rc_file}.tmp" "${rc_file}"

    # Build init block into a temp file (avoids $() stripping newlines)
    local init_tmp
    init_tmp="$(mktemp)"

    printf '# qwert\n' >> "${init_tmp}"
    printf 'export PATH="${HOME}/.qwert/bin:${PATH}"\n' >> "${init_tmp}"

    # fpath for zsh completions (must be before compinit)
    local shell_name
    shell_name="$(basename "${SHELL:-bash}")"
    if [ "${shell_name}" = "zsh" ]; then
        printf 'fpath=("${HOME}/.qwert/completions" $fpath)\n' >> "${init_tmp}"
    fi

    printf 'eval "$(qwert hook init)"\n' >> "${init_tmp}"
    printf '\n' >> "${init_tmp}"

    # Prepend init block
    cat "${init_tmp}" "${rc_file}" > "${rc_file}.tmp" && mv "${rc_file}.tmp" "${rc_file}"
    rm "${init_tmp}"
    ok "init hook added to top of ${rc_file}"

    printf '\neval "$(qwert hook end)"\n' >> "${rc_file}"
    ok "end hook added to bottom of ${rc_file}"
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

main() {
    # Parse args
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                QWERT_VERSION="$2"
                shift 2
                ;;
            *)
                die "unknown option: $1"
                ;;
        esac
    done

    printf "\n\033[1mqwert installer\033[0m\n"
    printf "%s\n\n" "───────────────"

    need curl
    need git

    ask_config_dir

    printf "\n"

    # Prefer pre-built binary; fall back to building from source if download fails
    if ! install_binary 2>/dev/null; then
        if command -v cargo &>/dev/null; then
            install_from_source
        else
            die "no pre-built binary available for this platform and cargo not found"
        fi
    fi

    install_recipes
    install_completions
    configure_shell

    printf "\n"
    ok "qwert installed successfully"
    info "Restart your shell or run: source ${rc_file:-~/.zshrc}"
    printf "\n"
}

main "$@"
