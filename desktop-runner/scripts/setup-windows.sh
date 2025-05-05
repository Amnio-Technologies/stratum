#!/usr/bin/env bash
set -e

echo "Installing MSYS2 packages..."

pacman -Syu --noconfirm
pacman -S --needed --noconfirm \
  mingw-w64-x86_64-gcc \
  mingw-w64-x86_64-cmake \
  mingw-w64-x86_64-pkg-config \
  mingw-w64-x86_64-make \
  mingw-w64-x86_64-SDL2 \
  unzip \
  curl \
  git

echo "Installing Rust (GNU toolchain)..."
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"

rustup default stable-x86_64-pc-windows-gnu

echo "Done. Make sure you're in a MinGW64 shell before running 'cargo build'."
