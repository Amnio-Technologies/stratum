#!/usr/bin/env python3

import shutil
import subprocess
import sys
from pathlib import Path
import multiprocessing

# -------- Configuration --------
PROJECT_ROOT = Path(__file__).parent.resolve()
FONT_C_GEN = "tools/generate_font_c_files.py"
FONT_H_GEN = "tools/generate_font_headers.py"
GENERATOR = "MinGW Makefiles"

ESP_IDF_PATH = Path.home() / "esp" / "esp-idf"
TOOLCHAIN_FILE = ESP_IDF_PATH / "tools" / "cmake" / "toolchain-esp32.cmake"
EXPORT_SH = Path.home() / "export-esp.sh"

# -------- Parse Args --------
target = "desktop"
build_type = "Debug"
args = sys.argv[1:]
if "--target" in args:
    t = args[args.index("--target") + 1]
    if t in ("desktop", "firmware"):
        target = t
if "--release" in args:
    build_type = "Release"

print(f"🔧 Building stratum-ui as a STATIC LIBRARY ({target.upper()}, {build_type})")

# -------- Firmware-only validation --------
if target == "firmware":
    if not TOOLCHAIN_FILE.exists():
        print(f"❌ Toolchain not found: {TOOLCHAIN_FILE}")
        sys.exit(1)
    if not EXPORT_SH.exists():
        print(f"❌ export-esp.sh not found: {EXPORT_SH}")
        sys.exit(1)

# -------- Generate Fonts --------
for script in (FONT_C_GEN, FONT_H_GEN):
    print(f"📁 Running {script}...")
    r = subprocess.run(["python3", script], cwd=PROJECT_ROOT)
    if r.returncode != 0:
        print(f"❌ {script} failed.")
        sys.exit(1)

# -------- Prepare Build Dir --------
build_dir = PROJECT_ROOT / "build" / target
if build_dir.exists():
    print("🧹 Cleaning previous build...")
    shutil.rmtree(build_dir)
build_dir.mkdir(parents=True)

# -------- Configure CMake --------
print("⚙️  Running CMake configuration...")
cmake_cmd = [
    "cmake",
    f"-DCMAKE_BUILD_TYPE={build_type}",
    f"-DSTRATUM_TARGET={target}",
    str(PROJECT_ROOT),
]

if target == "desktop":
    # desktop: inject generator
    cmake_cmd[1:1] = ["-G", GENERATOR]
    try:
        subprocess.run(cmake_cmd, cwd=build_dir, check=True)
    except subprocess.CalledProcessError:
        print("❌ CMake configuration failed.")
        sys.exit(1)

else:
    # firmware: toolchain + IDF env
    cmake_cmd.insert(1, f"-DCMAKE_TOOLCHAIN_FILE={TOOLCHAIN_FILE}")
    cmake_cmd.insert(2, f"-DIDF_TARGET=esp32")
    full = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(cmake_cmd)
    try:
        subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)
    except subprocess.CalledProcessError:
        print("❌ CMake configuration failed.")
        sys.exit(1)

# -------- Build --------
print("🏗️  Building stratum-ui...")
cpu = multiprocessing.cpu_count()
build_cmd = ["cmake", "--build", ".", "--", f"-j{cpu}"]

if target == "desktop":
    try:
        subprocess.run(build_cmd, cwd=build_dir, check=True)
    except subprocess.CalledProcessError:
        print("❌ Build failed.")
        sys.exit(1)

else:
    full = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(build_cmd)
    try:
        subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)
    except subprocess.CalledProcessError:
        print("❌ Build failed.")
        sys.exit(1)

print(f"✅ Build Complete! Output: {build_dir / 'libstratum-ui.a'}")
