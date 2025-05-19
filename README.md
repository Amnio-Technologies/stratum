# 🔋 What is Stratum?

**Stratum** is a smart, modular battery bank designed for hackers, engineers, and power users who demand more from their gear. Built under the amnIO brand, Stratum lets you **toollessly swap both ports and battery cells**, giving you full control over your portable power system — no screwdrivers, no bullshit.

Unlike traditional battery packs, Stratum is:

* 🔌 **Port-Swappable** – Add, remove, or upgrade output/input ports (USB-C, USB-A, DC barrel, etc.) with **snap-in modules**.
* 🔋 **Cell-Swappable** – Open the enclosure and drop in new 18650 cells, no soldering or teardown required.
* 🧠 **Smart** – Real-time diagnostics, per-cell health monitoring, and intelligent charge/discharge management via the onboard ESP32-S3.
* 🛡️ **Safe** – Includes active balancing, thermal monitoring, and hard cutoff protection for sketchy cells.
* 📟 **Readable** – Comes with a compact OLED or e-ink display for live stats: voltages, currents, cell health, and more.

Stratum is part of a broader vision at Amnio: making serious electronics tools **open, modular, and repairable** for anyone who's sick of black-box gear.

---

## ⚙️ System Requirements (for firmware devs)

To build and contribute to Stratum firmware:

* 🦀 **Rust** – Install via [rustup](https://rustup.rs/)
* 🧱 **CMake** – Required for native desktop builds, install via `winget install Kitware.CMake` or [cmake.org](https://cmake.org/download/)
* 🕹️ **SDL2** – Used for simulating the UI (`sdl2-sys` handles bindings, but you still need CMake installed)
