#!/usr/bin/env bash
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKGVER=$(grep '^version = ' "$REPO_DIR/Cargo.toml" | head -n1 | cut -d'"' -f2)

echo "🚀 Preparing DotDog AUR Release (v$PKGVER)..."

# 1. Publish dotdog-git
TMP_DIR=$(mktemp -d)
echo "📦 Cloning dotdog-git AUR repository..."
git clone ssh://aur@aur.archlinux.org/dotdog-git.git "$TMP_DIR"

echo "📄 Copying PKGBUILD and .SRCINFO for dotdog-git..."
cp "$REPO_DIR/aur/dotdog-git/PKGBUILD" "$REPO_DIR/aur/dotdog-git/.SRCINFO" "$TMP_DIR/"

cd "$TMP_DIR"
git branch -m master 2>/dev/null || true
git add PKGBUILD .SRCINFO
git commit -m "Update dotdog-git to $PKGVER" || true
git push -u origin master
cd "$REPO_DIR"
rm -rf "$TMP_DIR"

# 2. Publish dotmatrix-git (transitional)
TMP_DIR2=$(mktemp -d)
echo "📦 Cloning dotmatrix-git AUR repository..."
git clone ssh://aur@aur.archlinux.org/dotmatrix-git.git "$TMP_DIR2"

echo "📄 Copying PKGBUILD and .SRCINFO for dotmatrix-git..."
cp "$REPO_DIR/aur/dotmatrix-git/PKGBUILD" "$REPO_DIR/aur/dotmatrix-git/.SRCINFO" "$TMP_DIR2/"

cd "$TMP_DIR2"
git branch -m master 2>/dev/null || true
git add PKGBUILD .SRCINFO
git commit -m "Update dotmatrix-git to $PKGVER" || true
git push -u origin master
cd "$REPO_DIR"
rm -rf "$TMP_DIR2"

echo "🎉 DotDog v$PKGVER successfully published to AUR!"
