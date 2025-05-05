#!/usr/bin/env python3

import shutil
import subprocess
import sys
from pathlib import Path
import multiprocessing

# -------- Configuration --------
BUILD_DIR = "build"
FONT_C_FILE_GEN_SCRIPT = "tools/generate_font_c_files.py"
FONT_HEADER_GEN_SCRIPT = "tools/generate_font_headers.py"
GENERATOR = "MinGW Makefiles"
PROJECT_ROOT = Path(__file__).parent.resolve()

# -------- Parse command line arguments --------
build_type = "Debug"
if len(sys.argv) > 1 and sys.argv[1] == "--release":
    build_type = "Release"

print(f"🔧 Building stratum-ui as a STATIC LIBRARY ({build_type})...")

# -------- Run font c-file generator --------
print("📁 Generating font headers...")
result = subprocess.run(["python3", FONT_C_FILE_GEN_SCRIPT], cwd=PROJECT_ROOT)
if result.returncode != 0:
    print("❌ Font header c-file generation failed.")
    sys.exit(1)

# -------- Run font header generator --------
print("📁 Generating font headers...")
result = subprocess.run(["python3", FONT_HEADER_GEN_SCRIPT], cwd=PROJECT_ROOT)
if result.returncode != 0:
    print("❌ Font header generation failed.")
    sys.exit(1)

# -------- Clean build directory --------
build_path = PROJECT_ROOT / BUILD_DIR
if build_path.exists():
    print("🧹 Cleaning previous build...")
    shutil.rmtree(build_path)
build_path.mkdir()

# -------- Configure CMake --------
print("⚙️ Running CMake configuration...")
cmake_cmd = [
    "cmake",
    "-G", GENERATOR,
    f"-DCMAKE_BUILD_TYPE={build_type}",
    str(PROJECT_ROOT)
]
result = subprocess.run(cmake_cmd, cwd=build_path)
if result.returncode != 0:
    print("❌ CMake configuration failed.")
    sys.exit(1)

# -------- Build project --------
print("🏗️  Building stratum-ui...")
cpu_count = multiprocessing.cpu_count()
build_cmd = [
    "cmake", "--build", ".", "--", f"-j{cpu_count}"
]
result = subprocess.run(build_cmd, cwd=build_path)
if result.returncode != 0:
    print("❌ Build failed.")
    sys.exit(1)

print("✅ Build Complete!")
