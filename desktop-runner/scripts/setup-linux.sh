#!/bin/bash
set -e

echo "Installing Linux system dependencies..."

sudo apt update
sudo apt install -y \
  build-essential \
  cmake \
  pkg-config \
  libsdl2-dev \
  clang \
  libclang-dev \
  unzip \
  curl

echo "Installing Rust..."
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"

echo "Done. If you're using bindgen, ensure this is set in your shell config:"
echo 'export LIBCLANG_PATH=/usr/lib/llvm-14/lib'  # adjust for your LLVM version
