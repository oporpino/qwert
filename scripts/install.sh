#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="oporpino"
REPO_NAME="qwert"
REPO="https://github.com/${REPO_OWNER}/${REPO_NAME}"
API="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"

QWERT_BIN="/opt/qwert/bin"
QWERT_VERSION="${QWERT_VERSION:-}"

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

info() { printf "  \033[1m→\033[0m  %s\n" "$*"; }
ok()   { printf "  \033[32m✓\033[0m  %s\n" "$*"; }
die()  { printf "  \033[31m✗\033[0m  %s\n" "$*" >&2; exit 1; }

need() { command -v "$1" &>/dev/null || die "required: $1 not found"; }

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
                *)       die "unsupported Linux arch: ${arch}" ;;
            esac
            ;;
        *)
            die "unsupported OS: ${os}"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

main() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version) QWERT_VERSION="$2"; shift 2 ;;
            *)         die "unknown option: $1" ;;
        esac
    done

    printf "\n\033[1mqwert installer\033[0m\n"
    printf "%s\n\n" "───────────────"

    need curl
    need sudo

    local target version url tmp
    target="$(detect_target)"
    info "Platform: ${target}"

    if [ -n "${QWERT_VERSION}" ]; then
        version="${QWERT_VERSION}"
        info "Version: ${version} (pinned)"
    else
        info "Fetching latest release..."
        version="$(curl -fsSL "${API}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"
        [ -n "${version}" ] || die "could not determine latest release"
        info "Version: ${version}"
    fi

    url="${REPO}/releases/download/${version}/qwert-${target}"
    tmp="$(mktemp)"

    info "Downloading binary..."
    curl -fsSL --progress-bar "${url}" -o "${tmp}" || die "download failed: ${url}"

    sudo mkdir -p "${QWERT_BIN}"
    sudo mv "${tmp}" "${QWERT_BIN}/qwert"
    sudo chmod +x "${QWERT_BIN}/qwert"
    ok "Binary installed → ${QWERT_BIN}/qwert"

    printf "\n"
    sudo "${QWERT_BIN}/qwert" self install
}

main "$@"
