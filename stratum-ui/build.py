#!/usr/bin/env python3

import shutil
import subprocess
import sys
from pathlib import Path
import multiprocessing

# -------- Configuration --------
PROJECT_ROOT = Path(__file__).parent.resolve()
FONT_C_FILE_GEN_SCRIPT = "tools/generate_font_c_files.py"
FONT_HEADER_GEN_SCRIPT = "tools/generate_font_headers.py"

ESP_IDF_PATH = Path.home() / "esp" / "esp-idf"
TOOLCHAIN_FILE = ESP_IDF_PATH / "tools" / "cmake" / "toolchain-esp32.cmake"
EXPORT_SH = Path.home() / "export-esp.sh"  # from espup

# -------- Validate --------
if not TOOLCHAIN_FILE.exists():
    print(f"❌ Toolchain file not found: {TOOLCHAIN_FILE}")
    sys.exit(1)

if not EXPORT_SH.exists():
    print(f"❌ export-esp.sh not found: {EXPORT_SH}")
    sys.exit(1)

# -------- Parse Args --------
target = "desktop"
build_type = "Debug"

args = sys.argv[1:]
if "--target" in args:
    idx = args.index("--target") + 1
    if idx < len(args):
        target = args[idx]

if "--release" in args:
    build_type = "Release"

print(f"🔧 Building stratum-ui as a STATIC LIBRARY ({target.upper()}, {build_type})")

# -------- Generate Fonts --------
print("📁 Generating font sources...")
if subprocess.run(["python3", FONT_C_FILE_GEN_SCRIPT], cwd=PROJECT_ROOT).returncode != 0:
    print("❌ Font C file generation failed.")
    sys.exit(1)

if subprocess.run(["python3", FONT_HEADER_GEN_SCRIPT], cwd=PROJECT_ROOT).returncode != 0:
    print("❌ Font header generation failed.")
    sys.exit(1)

# -------- Build Dir --------
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
    str(PROJECT_ROOT)
]

if target == "firmware":
    cmake_cmd.insert(1, f"-DCMAKE_TOOLCHAIN_FILE={TOOLCHAIN_FILE}")
    cmake_cmd.insert(2, f"-DIDF_TARGET=esp32")
    cmake_env_cmd = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(cmake_cmd)
    result = subprocess.run(["bash", "-c", cmake_env_cmd], cwd=build_dir)
else:
    result = subprocess.run(cmake_cmd, cwd=build_dir)

if result.returncode != 0:
    print("❌ CMake configuration failed.")
    sys.exit(1)

# -------- Build --------
print("🏗️  Building stratum-ui...")
cpu_count = multiprocessing.cpu_count()
build_cmd = ["cmake", "--build", ".", "--", f"-j{cpu_count}"]

if target == "firmware":
    build_env_cmd = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(build_cmd)
    result = subprocess.run(["bash", "-c", build_env_cmd], cwd=build_dir)
else:
    result = subprocess.run(build_cmd, cwd=build_dir)

if result.returncode != 0:
    print("❌ Build failed.")
    sys.exit(1)

print(f"✅ Build Complete! Output: build/{target}/libstratum-ui.a")
