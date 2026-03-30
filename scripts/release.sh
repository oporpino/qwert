#!/bin/bash
set -e

# Called by .commons/scripts/make/core/release.sh before tagging.
# Usage: scripts/release.sh <version>  (e.g. v0.1.29)

VERSION="${1:?version argument required (e.g. v0.1.29)}"
VERSION_BARE="${VERSION#v}"

sed -i.bak "s/^version = .*/version = \"${VERSION_BARE}\"/" Cargo.toml && rm Cargo.toml.bak
git add Cargo.toml
git commit --amend --no-edit
git push origin main --force-with-lease
