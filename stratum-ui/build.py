#!/usr/bin/env python3

import shutil
import subprocess
import sys
import time
from pathlib import Path
import multiprocessing
import argparse
import io

# Ensure proper UTF-8 stdout
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

# -------- Configuration --------
PROJECT_ROOT = Path(__file__).parent.resolve()
FONT_C_GEN = "tools/generate_font_c_files.py"
FONT_H_GEN = "tools/generate_font_headers.py"
GENERATOR = "MinGW Makefiles"

ESP_IDF_PATH = Path.home() / "esp" / "esp-idf"
TOOLCHAIN_FILE = ESP_IDF_PATH / "tools" / "cmake" / "toolchain-esp32.cmake"
EXPORT_SH = Path.home() / "export-esp.sh"

# -------- Argument Parsing --------
parser = argparse.ArgumentParser()
parser.add_argument("--dynamic", action="store_true", help="Build a shared library")
parser.add_argument("--no-cache", action="store_true", help="Force rebuild")
parser.add_argument("--target", type=str, default="desktop", choices=["desktop", "firmware"])
parser.add_argument("--release", action="store_true", help="Use Release build")
parser.add_argument("--output-name", type=str, help="Override output library name (without extension)")
args = parser.parse_args()

target = args.target
build_type = "Release" if args.release else "Debug"
is_dynamic = args.dynamic
no_cache = args.no_cache
output_name = args.output_name

lib_type_str = "DYNAMIC SHARED LIBRARY" if is_dynamic else "STATIC LIBRARY"
print(f"🔧 Building stratum-ui as a {lib_type_str} ({target.upper()}, {build_type})")

# -------- Start Timer --------
build_start = time.time()

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
    cmd = ["python3", script]
    if no_cache:
        cmd.append("--no-cache")
    r = subprocess.run(cmd, cwd=PROJECT_ROOT)
    if r.returncode != 0:
        print(f"❌ {script} failed.")
        sys.exit(1)

# -------- Prepare Build Dir --------
build_dir = PROJECT_ROOT / "build" / target
if no_cache and build_dir.exists():
    print("🧹 Cleaning previous build...")
    shutil.rmtree(build_dir)
build_dir.mkdir(parents=True, exist_ok=True)

# -------- Configure CMake --------
print("⚙️  Running CMake configuration...")
cmake_cmd = [
    "cmake",
    f"-DCMAKE_BUILD_TYPE={build_type}",
    f"-DSTRATUM_TARGET={target}",
    f"-DSTRATUM_BUILD_DYNAMIC={'ON' if is_dynamic else 'OFF'}",
    str(PROJECT_ROOT),
]

if output_name:
    cmake_cmd.insert(-1, f"-DSTRATUM_OUTPUT_NAME={output_name}")

if target == "desktop":
    cmake_cmd[1:1] = ["-G", GENERATOR]
    try:
        subprocess.run(cmake_cmd, cwd=build_dir, check=True)
    except subprocess.CalledProcessError:
        print("❌ CMake configuration failed.")
        sys.exit(1)
else:
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

# -------- End Timer --------
elapsed = time.time() - build_start
minutes = int(elapsed // 60)
seconds = int(elapsed % 60)

# -------- Output Summary --------
if output_name:
    base_name = output_name
else:
    if is_dynamic:
        ext = ".dll" if sys.platform == "win32" else ".dylib" if sys.platform == "darwin" else ".so"
        base_name = f"libstratum-ui{ext}"
    else:
        base_name = "libstratum-ui.a"

print(f"✅ Build Complete! Output: {build_dir / base_name}")
print(f"⏱️ Build finished in {minutes}m {seconds}s")

# -------- Clean up unneeded import libraries (Windows-only) --------
if sys.platform == "win32" and is_dynamic:
    import_lib = build_dir / f"lib{base_name}.dll.a"
    print(import_lib);
    
    if import_lib.exists():
        import_lib.unlink()
        print(f"🧹 Removed import library: {import_lib}")