# amnio-ui (C LVGL Backend)

This is the **amnIO LVGL UI**, which provides a C-based UI implementation that **desktop-runner** and **embedded-runner** link against.

---

## **🛠 Prerequisites**
Ensure the following are installed:

- **CMake** (Minimum version: `3.15`)
- **MinGW-w64 (GCC)**
- **MSYS2 Terminal** (or equivalent on Linux/macOS)

---

## **🚀 Build Instructions**
Run the following in **MSYS2 MinGW64** (or your shell of choice):

```sh
cd amnio-ui
mkdir build && cd build
cmake .. -G "MinGW Makefiles"
mingw32-make
```

or simply run the pre-made script:

```sh
./build.sh
```

This will generate:

- `libamnio-ui.dll` → The shared library (`.so` on Linux, `.dylib` on macOS)
- `libamnio-ui.dll.a` → The import library for linking

---

## **📝 Output Files**
After building, you'll find:
```
amnio-ui/build/
├── libamnio-ui.dll       # DLL for Rust
├── libamnio-ui.dll.a     # Import library
└── CMakeCache.txt        # Build configuration
```

---

## **⚠️ Troubleshooting**
### **CMake Error: "Cannot find source file"**
If CMake fails with missing source file errors, **try deleting old build cache**:
```sh
cd amnio-ui/build
rm -rf CMakeCache.txt CMakeFiles/
cmake .. -G "MinGW Makefiles"
mingw32-make
```

### **"DLL not found" when running `desktop-runner`**
Ensure that **`libamnio-ui.dll` is copied to `target/debug/`**:
```sh
cp amnio-ui/build/libamnio-ui.dll target/debug/amnio-ui.dll
```
---

## **✅ Next Steps**
Now, you can build `desktop-runner` to run the UI:
```sh
cd ../desktop-runner
cargo build
cargo run
```
