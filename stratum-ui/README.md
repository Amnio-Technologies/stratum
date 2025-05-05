# stratum-ui (C LVGL Backend)

This is the **amnIO LVGL UI**, which provides a C-based UI implementation that **desktop-runner** and **embedded-runner** link against.

---

## **🛠 Prerequisites**
Ensure the following are installed:

- **CMake** (Minimum version: `3.15`)
- **MinGW-w64 (GCC)**
- **MSYS2 Terminal** (or equivalent on Linux/macOS)
- **lv_font_conv** (see below for installation instructions via NPM)
---

## 🧰 Font Conversion Setup (lv_font_conv)
To convert TTF fonts to C files for LVGL, you need lv_font_conv, which requires Node.js. Install `lv_font_conv` by running.
```npm i -g lv_font_conv```

## **Build Instructions**
Run the following in **MSYS2 MinGW64** (or your shell of choice):

```sh
cd stratum-ui
mkdir build && cd build
cmake .. -G "MinGW Makefiles"
mingw32-make
```

or simply run the pre-made script:

```sh
./build.sh
```

This will generate:

- `libstratum-ui.dll` → The shared library (`.so` on Linux, `.dylib` on macOS)
- `libstratum-ui.dll.a` → The import library for linking

---

## **📝 Output Files**
After building, you'll find:
```
stratum-ui/build/
├── libstratum-ui.dll       # DLL for Rust
├── libstratum-ui.dll.a     # Import library
└── CMakeCache.txt        # Build configuration
```

---

## **⚠️ Troubleshooting**
### **CMake Error: "Cannot find source file"**
If CMake fails with missing source file errors, **try deleting old build cache**:
```sh
cd stratum-ui/build
rm -rf CMakeCache.txt CMakeFiles/
cmake .. -G "MinGW Makefiles"
mingw32-make
```

### **"DLL not found" when running `desktop-runner`**
Ensure that **`libstratum-ui.dll` is copied to `target/debug/`**:
```sh
cp stratum-ui/build/libstratum-ui.dll target/debug/stratum-ui.dll
```
---

## **Next Steps**
Now, you can build `desktop-runner` to run the UI:
```sh
cd ../desktop-runner
cargo build
cargo run
```
