# What is amnIO?

amnIO is a modular, open-source electronics lab instrument designed to provide students, makers, and engineers with a portable, cloud-connected alternative to traditional lab equipment. It combines the functionality of an oscilloscope, waveform generator, and power supply into a compact, user-friendly system with hot-swappable modules. Unlike conventional lab tools, amnIO plans to integrate modern UI design, cloud storage, and real-time collaboration features, making it easier to use in university labs, maker spaces, and personal workbenches.

## 🛠 System Requirements

To build this project, you need:

- **Rust** (Install via [rustup](https://rustup.rs/))
- **CMake** (Required for `desktop-runner`, install via `winget install Kitware.CMake` or [CMake downloads](https://cmake.org/download/))
- **SDL2** (Handled by `sdl2-sys`, but requires CMake)
