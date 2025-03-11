# desktop-runner (amnIO Debug UI)

This is the **amnIO UI Debugger**, a Rust-based application that **renders the UI and integrates LVGL**.

---

## **🛠 Prerequisites**
Ensure you have:

- **Rust (stable)**
- **cargo** (comes with Rust)
- **CMake** (for `amnio-ui`)
- **MinGW-w64** (for building C code)
- **SDL2** (Ensure `sdl2.dll` is present in `target/debug`)

---

## **🚀 Build & Run**
First, **build `amnio-ui`** (C LVGL backend):
```sh
cd ../amnio-ui
mkdir build && cd build
cmake .. -G "MinGW Makefiles"
mingw32-make
```

Then, **build the Rust UI Debugger**:
```sh
cd ../../desktop-runner
cargo build
```

---

## **🏃 Running the Debugger**
```sh
cargo run
```
**⚡️ Important:** If you see `STATUS_DLL_NOT_FOUND`, ensure `amnio-ui.dll` is present in `target/debug/`:
```sh
cp ../amnio-ui/build/libamnio-ui.dll ../target/debug/amnio-ui.dll
```

---

## **🛠 Build System Notes**
### **Linking to amnio-ui (DLL)**
Rust needs to find `amnio-ui.dll`. Your `build.rs` already **copies the DLL automatically**, but if linking fails:

1. **Check if the DLL exists:**
   ```sh
   ls target/debug/amnio-ui.dll
   ```
   If missing, manually copy it:
   ```sh
   cp ../amnio-ui/build/libamnio-ui.dll target/debug/amnio-ui.dll
   ```

2. **Check Rust linking flags**
   `build.rs` tells Rust to link against `amnio-ui`:
   ```rust
   println!("cargo:rustc-link-search=native=C:/Users/erick/Documents/amnio/ui/amnio-ui/build");
   println!("cargo:rustc-link-lib=dylib=amnio-ui");
   ```

---

## **✅ Next Steps**
1. **Make sure `amnio-ui` builds first.**
2. **Check that the DLL is copied to `target/debug/`.**
3. **Run `cargo run` and test the UI!** 🚀
