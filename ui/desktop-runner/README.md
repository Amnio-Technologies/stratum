# amnIO Desktop Runner  

This module runs the **amnIO UI on a desktop environment** using **SDL2 + egui**.  

## 📌 Requirements  

To build `desktop-runner`, install:  

1. **Rust** → [Install via rustup](https://rustup.rs/)  
2. **CMake** (Required for `sdl2-sys`)  
   - Install via **Winget**:  
     ```sh
     winget install Kitware.CMake
     ```
   - Or manually from [CMake downloads](https://cmake.org/download/)  
3. **SDL2** (Managed by `sdl2-sys`, but requires CMake)  

**⚠️ Troubleshooting: Missing Build Tools on Windows**  
If you run into build issues, ensure that:  
- **Visual Studio Installer is active** and the required C++ build tools are installed.  
- You have **MSVC (Microsoft C++ Build Tools) installed** via Visual Studio.  
- You restart your terminal after installing dependencies.  

## 🚀 Running the Desktop UI  
After installing dependencies, build & run:  

```sh
cargo run -p desktop-runner
```  