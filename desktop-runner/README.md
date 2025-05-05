Here’s a **unified README** that supports both **Windows (MSYS2 + GNU toolchain)** and **Linux/WSL**, cleanly organized so devs can follow either track:

---

# desktop-runner (amnIO Debug UI)

This is the **amnIO UI Debugger**, a Rust-based application that **renders the UI and integrates LVGL**.
It supports both **Windows (MSYS2)** and **Linux/WSL** setups.

---

## 🧰 Prerequisites

### ✅ Universal:

* **Rust (stable)**: [`https://rustup.rs`](https://rustup.rs)
* **cargo**: (comes with Rust)
* **CMake**: required for `stratum-ui` C build
* **LVGL C backend**: lives in `../stratum-ui`

---

### 🪟 For **Windows (MSYS2/GNU Toolchain)**:

* [MSYS2](https://www.msys2.org)
* `pacman -S mingw-w64-x86_64-toolchain make cmake`
* SDL2: place `sdl2.dll` in `target/debug/` manually or use package manager
* Run in **MSYS2 MinGW 64-bit shell**
* **Rust target override**:

  ```sh
  rustup override set stable-x86_64-pc-windows-gnu
  ```

---

### 🐧 For **Linux/WSL**:

Install system dependencies:

```sh
sudo apt update
sudo apt install -y clang libclang-dev cmake pkg-config libsdl2-dev
```

Set `LIBCLANG_PATH` if you hit libclang errors:

```sh
export LIBCLANG_PATH=/usr/lib/llvm-14/lib  # adjust version if needed
```

Make sure Rust is installed:

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
```

---

## 🔨 Build Steps

### 1. Build the C UI backend (`stratum-ui`)

```sh
cd ../stratum-ui
./build.sh
```

### 2. Build the Rust Debugger

```sh
cd ../../desktop-runner
cargo build
```

---

## ▶️ Run the Debugger

```sh
cargo run
```

If you see `STATUS_DLL_NOT_FOUND` (Windows only), manually copy:

```sh
cp ../stratum-ui/build/libstratum-ui.dll target/debug/stratum-ui.dll
```

---

## 🧱 Build System Notes

### Linking to `stratum-ui`

Rust looks for the C backend via `build.rs`. If linking fails:

1. Check if the DLL or static lib exists:

   ```sh
   ls ../stratum-ui/build/libstratum-ui.{dll,a}
   ```

2. Rust linking flags (in `build.rs`):

   ```rust
   println!("cargo:rustc-link-search=native=../stratum-ui/build");
   println!("cargo:rustc-link-lib=dylib=stratum-ui");  // or static=stratum-ui
   ```

---

## ✅ Next Steps

* Ensure `stratum-ui` builds before anything else.
* Confirm the C library is copied or linked correctly.
* Run `cargo run` to launch the amnIO UI Debugger.