#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

BUMP_TYPE=""
AUTO_APPROVE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --patch)
      BUMP_TYPE="1"
      shift
      ;;
    --minor)
      BUMP_TYPE="2"
      shift
      ;;
    --major)
      BUMP_TYPE="3"
      shift
      ;;
    --auto-approve)
      AUTO_APPROVE=true
      shift
      ;;
    *)
      printf "${RED}Unknown option: $1${NC}\n"
      printf "Usage: $0 [--patch|--minor|--major] [--auto-approve]\n"
      exit 1
      ;;
  esac
done

printf "${BLUE}> Release Creator${NC}\n\n"

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
  printf "${RED}  - [error] You must be on 'main' branch (current: %s)${NC}\n" "$CURRENT_BRANCH"
  exit 1
fi
printf "${GREEN}  - On main branch${NC}\n"

printf "\n> Fetching latest from origin...\n"
git fetch origin main --tags --prune

LOCAL=$(git rev-parse main)
REMOTE=$(git rev-parse origin/main)

if [ "$LOCAL" != "$REMOTE" ]; then
  printf "${RED}  - [error] Local main is not up to date with origin/main${NC}\n"
  printf "    Run: git pull origin main\n"
  exit 1
fi
printf "${GREEN}  - Main is up to date${NC}\n"

CURRENT_VERSION=$(git tag --sort=-version:refname | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -1)
if [ -z "$CURRENT_VERSION" ]; then
  CURRENT_VERSION="v0.0.0"
fi
printf "\n> Current version: ${YELLOW}%s${NC}\n" "$CURRENT_VERSION"

VERSION=${CURRENT_VERSION#v}
IFS='.' read -ra VERSION_PARTS <<< "$VERSION"
MAJOR=${VERSION_PARTS[0]:-0}
MINOR=${VERSION_PARTS[1]:-0}
PATCH=${VERSION_PARTS[2]:-0}

if [ -z "$BUMP_TYPE" ]; then
  printf "\n> What type of release?\n"
  printf "  1) ${GREEN}patch${NC} - Bug fixes (e.g., %s → v%s.%s.%s)\n" "$CURRENT_VERSION" "$MAJOR" "$MINOR" "$((PATCH + 1))"
  printf "  2) ${YELLOW}minor${NC} - New features (e.g., %s → v%s.%s.0)\n" "$CURRENT_VERSION" "$MAJOR" "$((MINOR + 1))"
  printf "  3) ${RED}major${NC} - Breaking changes (e.g., %s → v%s.0.0)\n" "$CURRENT_VERSION" "$((MAJOR + 1))"
  printf "  q) quit\n\n"
  read -p "Enter choice [1-3/q]: " BUMP_TYPE
fi

case $BUMP_TYPE in
  1)
    PATCH=$((PATCH + 1))
    BUMP_NAME="patch"
    ;;
  2)
    MINOR=$((MINOR + 1))
    PATCH=0
    BUMP_NAME="minor"
    ;;
  3)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    BUMP_NAME="major"
    ;;
  q|Q)
    printf "${YELLOW}Bye!${NC}\n"
    exit 0
    ;;
  *)
    printf "${RED}  - [error] Invalid choice${NC}\n"
    exit 1
    ;;
esac

NEW_VERSION="v$MAJOR.$MINOR.$PATCH"

printf "\n> Version bump (%s): ${YELLOW}%s${NC} → ${GREEN}%s${NC}\n" "$BUMP_NAME" "$CURRENT_VERSION" "$NEW_VERSION"

if [ "$AUTO_APPROVE" = false ]; then
  printf "\n"
  read -p "Create and push tag $NEW_VERSION? [y/N]: " CONFIRM
  if [[ ! $CONFIRM =~ ^[Yy]$ ]]; then
    printf "${YELLOW}Aborted${NC}\n"
    exit 0
  fi
fi

printf "\n> Creating tag %s...\n" "$NEW_VERSION"
git tag "$NEW_VERSION"
printf "${GREEN}  - Tag created locally${NC}\n"

printf "> Pushing tag to origin...\n"
git push origin "$NEW_VERSION"
printf "${GREEN}  - Tag %s pushed${NC}\n" "$NEW_VERSION"

printf "> Updating latest tag...\n"
git tag -f latest
git push origin latest --force
printf "${GREEN}  - Tag latest updated${NC}\n"

printf "\n${GREEN}> Release %s done!${NC}\n" "$NEW_VERSION"
