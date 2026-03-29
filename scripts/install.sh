#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="gporpino"
REPO_NAME="qwert"
REPO="https://github.com/${REPO_OWNER}/${REPO_NAME}"
API="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"

QWERT_HOME="${HOME}/.qwert"
QWERT_BIN="${QWERT_HOME}/bin"
QWERT_RECIPES="${QWERT_HOME}/recipes"

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

info()    { printf "  \033[1m→\033[0m  %s\n" "$*"; }
ok()      { printf "  \033[32m✓\033[0m  %s\n" "$*"; }
warn()    { printf "  \033[33m!\033[0m  %s\n" "$*"; }
die()     { printf "  \033[31m✗\033[0m  %s\n" "$*" >&2; exit 1; }

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
# install binary
# ---------------------------------------------------------------------------

install_binary() {
    local target version download_url tmp

    target="$(detect_target)"
    info "Platform: ${target}"

    # Fetch latest release version from GitHub API
    info "Fetching latest release..."
    version="$(curl -fsSL "${API}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"
    [ -n "${version}" ] || die "could not determine latest release version"
    info "Latest version: ${version}"

    download_url="${REPO}/releases/download/${version}/qwert-${target}"
    tmp="$(mktemp)"

    info "Downloading binary..."
    curl -fsSL --progress-bar "${download_url}" -o "${tmp}" \
        || die "download failed: ${download_url}"

    mkdir -p "${QWERT_BIN}"
    mv "${tmp}" "${QWERT_BIN}/qwert"
    chmod +x "${QWERT_BIN}/qwert"
    ok "Binary installed → ${QWERT_BIN}/qwert"
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
# install recipes
# ---------------------------------------------------------------------------

install_recipes() {
    info "Installing recipes..."
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    git clone --depth 1 "${REPO}.git" "${tmp_dir}" \
        || die "could not fetch recipes"

    mkdir -p "${QWERT_RECIPES}"
    # Remove old flat .toml files if present
    find "${QWERT_RECIPES}" -maxdepth 1 -name "*.toml" -delete 2>/dev/null || true
    cp -r "${tmp_dir}/recipes/"* "${QWERT_RECIPES}/"
    rm -rf "${tmp_dir}"
    ok "Recipes installed → ${QWERT_RECIPES}"
}

# ---------------------------------------------------------------------------
# configure shell
# ---------------------------------------------------------------------------

configure_shell() {
    local rc_file path_line

    if [ -f "${HOME}/.zshrc" ]; then
        rc_file="${HOME}/.zshrc"
    elif [ -f "${HOME}/.bashrc" ]; then
        rc_file="${HOME}/.bashrc"
    else
        warn "no .zshrc or .bashrc found — add ${QWERT_BIN} to your PATH manually"
        return
    fi

    path_line='export PATH="${HOME}/.qwert/bin:${PATH}"'

    if grep -qF '.qwert/bin' "${rc_file}"; then
        ok "PATH already configured in ${rc_file}"
    else
        echo "" >> "${rc_file}"
        echo "# qwert" >> "${rc_file}"
        echo "${path_line}" >> "${rc_file}"
        ok "PATH configured in ${rc_file}"
    fi
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

main() {
    printf "\n\033[1mqwert installer\033[0m\n"
    printf "%s\n\n" "───────────────"

    need curl
    need git

    # Try downloading a pre-built binary; fall back to building from source
    if ! install_binary 2>/dev/null; then
        warn "no pre-built binary available for this platform"
        install_from_source
    fi

    install_recipes
    configure_shell

    printf "\n"
    ok "qwert installed successfully"
    info "Restart your shell or run: source ~/.zshrc"
    printf "\n"
}

main "$@"
