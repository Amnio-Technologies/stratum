#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Installing required packages..."

# Core toolchain (from official MSYS2 repo)
pacman -Syu --needed --noconfirm \
  base-devel \
  mingw-w64-x86_64-clang \
  mingw-w64-x86_64-cmake \
  mingw-w64-x86_64-gdb-multiarch \
  mingw-w64-x86_64-make \
  mingw-w64-x86_64-nodejs

# Git-based packages (need pacman-git or an AUR-like helper like `paru`, `yay`)
pacman -Syu --needed --noconfirm \
  git \
  mingw-w64-x86_64-libmangle-git \
  mingw-w64-x86_64-tools-git \
  mingw-w64-x86_64-winstorecompat-git

echo "📦 Installing global npm tools..."
npm install -g lv_font_conv

echo "🦀 Installing Rust toolchain..."
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
  echo '⚠️  You may need to restart your shell or add this to your profile:'
  echo '    export PATH="$HOME/.cargo/bin:$PATH"'
else
  echo "✅ Rust already installed via rustup"
fi

echo "✅ Environment setup complete."
